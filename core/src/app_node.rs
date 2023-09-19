use crate::db::NodeDB;
use crate::errors::Error;
use crate::state::VmState;
use crate::traits::StateMachine;
use crate::traits::TxHasher;
use crate::types::AggregatedBatch;
use crate::types::BatchWithProof;
use crate::types::BatchHeader;
use crate::types::TransactionReceipt;
use crate::types::TransactionWithReceipt;
use avail::service::{DaProvider as AvailDaProvider, DaServiceConfig};
use primitive_types::U256;
use risc0_zkp::core::digest::Digest;
use risc0_zkvm::{
    default_executor_from_elf,
    serde::{from_slice, to_vec},
    ExecutorEnv, SessionReceipt,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_slice as from_json_slice, to_vec as to_json_vec};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;
use sparse_merkle_tree::MerkleProof;
use std::marker::PhantomData;
use std::time::SystemTime;
use std::time::Duration;
use std::mem;

//Below imports for HTTP server.
use actix_web::error;
use actix_web::rt::System;
use actix_web::HttpResponse;
use actix_web::{get, web, App, HttpServer, Responder};
use reqwest;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppNodeConfig {
    //TODO: prover to be initiated only if mode set to true.
    pub prover_mode: bool,
    pub light_client_url: String,
    pub node_client_url: String,
    //TODO: Safer strategy to load seed so it is not accidentally revealed.
    pub seed: String,
    pub app_id: u32,
}

pub struct AppNode<V: Clone, T: Clone + DeserializeOwned + Serialize, S: StateMachine<V, T>> {
    state_machine: Arc<Mutex<S>>,
    db: Arc<Mutex<NodeDB>>,
    da_service: AvailDaProvider,
    chain: AppChain,
    zkvm_elf: Box<[u8]>,
    zkvm_id: Digest,
    phantom_v: PhantomData<V>,
    tx_pool: Arc<Mutex<Vec<T>>>,
}

impl<
        V: Serialize + DeserializeOwned + Clone,
        T: Clone + DeserializeOwned + Serialize + TxHasher,
        S: StateMachine<V, T>,
    > AppNode<V, T, S>
{   
    pub fn clone(&self) -> Self {
        Self {
            state_machine: self.state_machine.clone(),
            db: self.db.clone(),
            da_service: self.da_service.clone(),
            chain: self.chain.clone(),
            zkvm_elf: self.zkvm_elf.clone(),
            zkvm_id: self.zkvm_id.clone(),
            phantom_v: PhantomData,
            tx_pool: self.tx_pool.clone()
        }
    }

    pub async fn new(
        config: AppNodeConfig,
        zkvm_elf: &[u8],
        zkvm_id: impl Into<Digest>,
        chain: AppChain,
    ) -> Self {
        let node_db = NodeDB::from_path(String::from("./node_db"));
        let last_state_root: H256 = match node_db.get::<BatchHeader>(b"last_batch_header") {
            Ok(Some(i)) => i.state_root.clone(),
            Ok(None) => H256::zero(),
            Err(e) => panic!("Could not start node. {:?}", e),
        };
        let state_machine = Arc::new(Mutex::new(S::new(last_state_root)));
        let da_service = AvailDaProvider::new(DaServiceConfig {
            node_client_url: config.node_client_url,
            light_client_url: config.light_client_url,
            seed: config.seed,
            app_id: config.app_id,
        }).await;

        Self {
            state_machine,
            db: Arc::new(Mutex::new(node_db)),
            da_service,
            chain,
            zkvm_elf: zkvm_elf.into(),
            zkvm_id: zkvm_id.into(),
            phantom_v: PhantomData,
            tx_pool: Arc::new(Mutex::new(vec![]))
        }
    }

    //TODO: Complete implementation.
    // pub async fn sync(&mut self) -> Result<(), Error> {
    //     let start_height = 283562;
    //     let light_client_url = "http://127.0.0.1:8000".to_string();
    //     // Initialize the Avail service using the DaService interface
    //     let da_service = AvailDaProvider::new(DaServiceConfig {
    //         node_client_url: "wss://kate.avail.tools:443/ws".to_string(),
    //         light_client_url,
    //         seed: String::from("demo_seed"), 
    //         app_id: 7,
    //     }).await;

    //     for height in start_height.. {
    //         let filtered_block = match da_service.get_finalized_at(height).await {
    //             Ok(i) => i,
    //             Err(e) => panic!("{}", e.to_string()),
    //         };

    //         let batch: Option<Batch<T>> = match filtered_block.transactions.is_empty() {
    //             true => None,
    //             false => from_json_slice(&filtered_block.transactions[0].blob()).unwrap(),
    //         };

    //         match batch {
    //             None => println!("no batches in block"),
    //             Some(i) => println!("Found batch."),
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn run(&self) -> Result<(), Error> {
        loop {
            let mut tx_pool = self.tx_pool.lock().await;

            while !tx_pool.is_empty() {
                let last_state_root: H256 = {
                    let db = self.db.lock().await;

                    match db.get::<BatchHeader>(b"last_batch_header") {
                    Ok(Some(i)) => i.state_root.clone(),
                    Ok(None) => H256::zero(),
                    Err(e) => panic!("Could not start node. {:?}", e),
                    }
                }; 

                match self.execute_batch(tx_pool[0].clone()).await {
                    Ok(()) => (), 
                    Err(e) => {
                        let mut state_machine = self.state_machine.lock().await;

                        match state_machine.revert(last_state_root) {
                            Ok(()) => (), 
                            Err(e) => panic!("Reverting state failed. Need to restart node."),
                        };
                    }
                }

                //TODO: Only remove if the transaction failed 
                //due to state transition error. (or was successful)
                tx_pool.remove(0);
                continue;
            }

            //Make this configurable.
            tokio::time::sleep(Duration::from_secs(10)).await;
        } 
    }

    pub async fn execute_batch(&self, call_params: T) -> Result<(), Error> {
        let nexus_url = "http://127.0.0.1:8080/current-batch";
        let now = SystemTime::now();
        let last_batch_number: u64 = {
            let db = self.db.lock().await;

            match &db.get::<BatchHeader>(b"last_batch_header") {
            Ok(Some(i)) => i.batch_number,
            Ok(None) => 0,
            Err(e) => panic!("Could not start node. {:?}", e),
            }
        };
        //TODO: Add proper error handling below by removing unwrap and store last
        //batch in memory.
        let aggregated_proof: AggregatedBatch =
            reqwest::get(nexus_url).await.unwrap().json().await.unwrap();

        let mut state_machine = self.state_machine.lock().await;

        //TODO: Below should be replaced with a loop to execute a list of transactions.
        let (state_update, receipt) = state_machine
            .execute_tx(call_params.clone(), aggregated_proof.clone())
            .unwrap();

        // println!(
        //     "Pre state: {:?}, Post state: {:?}",
        //     &state_update.pre_state_root, &state_update.post_state_root
        // );

        //Note: Have to do this weird construction as tokio spawn complains that 
        //env is not dropped before an async operation below so is not thread safe.
        let serialized = {
            let mut exec = {
                let env = ExecutorEnv::builder()
                    .add_input(&to_vec(&call_params).unwrap())
                    .add_input(&to_vec(&state_update).unwrap())
                    .add_input(&to_vec(&(last_batch_number + 1)).unwrap())
                    .add_input(&to_vec(&aggregated_proof).unwrap())
                    .build()
                    .unwrap();

                    // Next, we make an executor, loading the (renamed) ELF binary.
                    default_executor_from_elf(env, &self.zkvm_elf).unwrap()
            };

            // Run the executor to produce a session.
            let session = exec.run().unwrap();
            let segments = session.resolve().unwrap();

            let cycles = segments
                .iter()
                .fold(0, |acc, segment| acc + (1 << segment.po2));

            println!("Executed, cycles: {}k", cycles / 1024);
            let session_receipt = session.prove().unwrap();

                println!("Session executed in zkvm with ID {:?}", &self.zkvm_id);
                session_receipt.verify(self.zkvm_id).unwrap();
            
            //TODO: Might not need to be deserialized, and need to remove unwrap.
            let batch_header: BatchHeader = from_slice(&session_receipt.journal).unwrap();
            let transaction_with_receipt = TransactionWithReceipt {
                transaction: call_params.clone(),
                receipt: receipt.clone(),
            };

            to_json_vec(&BatchWithProof {
                header: batch_header,
                proof: session_receipt,
                transaction_with_receipts: vec![transaction_with_receipt.clone()],
            })
            .unwrap()
        };

        match now.elapsed() {
            Ok(elapsed) => {
                // it prints '2'
                println!(
                    "execution done, time elapsed: {}s, tx count: {}, tx hash: {:?}",
                    elapsed.as_secs(),
                    last_batch_number + 1,
                    call_params.to_h256()
                );
            }
            Err(e) => {
                // an error occurred!
                println!("Error: {e:?}");
            }
        }

        // NEED TO  REWRITE BELOW.
        // let client = reqwest::Client::new();
        // let url = "http://localhost:8080/submit-batch"; // Change this to your server's URL

        // let response = client
        //     .post(url)
        //     .body(serialized)
        //     .header("Content-Type", "application/json")
        //     .send()
        //     .await
        //     .unwrap();

        // let status = response.status();

        // println!("Batch submit call resovled with status: {:?}", status);

        match self.da_service.send_transaction(&serialized).await {
            Ok(i) => (), 
            //Change from default error.
            Err(e) => return Err(Error::default()),
        };

        //Add batch header.
        let db = self.db.lock().await;

        match &db.put(
            b"last_batch_header",
            &BatchHeader {
                pre_state_root: state_update.pre_state_root.clone(),
                state_root: state_update.post_state_root.clone(),
                //TODO: Change both below to hash list of transactions and
                //not single one.
                transactions_root: call_params.to_h256(),
                receipts_root: receipt.to_h256(),
                batch_number: last_batch_number + 1,
            },
        ) {
            Ok(()) => (),
            Err(e) => return Err(e.clone()),
        };

        let transaction_with_receipt = TransactionWithReceipt {
            transaction: call_params.clone(),
            receipt: receipt.clone(),
        };
        
        //TODO: Add tx list. For now only one transaction.
        match &db.put(
            call_params.to_h256().as_slice(),
            &transaction_with_receipt,
        ) {
            Ok(()) => (),
            Err(e) => return Err(e.clone()),
        }

        Ok(())
    }

    pub async fn add_to_tx_pool(&self, tx: T) -> () {
        let mut tx_pool = self.tx_pool.lock().await;

        tx_pool.push(tx)
    }

    pub async fn get_state_with_proof(&self, key: &H256) -> Result<(V, MerkleProof), Error> {
        let state_machine = self.state_machine.lock().await;

        state_machine.get_state_with_proof(key)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct StateQuery {
  key: String,
}
fn hex_string_to_u8_array(hex_string: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_string).unwrap();
    
    if bytes.len() != 32 {
        panic!("Hexadecimal string must represent exactly 32 bytes");
    }
  
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
  
    array
}
async fn get_state_with_proof<V, T, S>(
    service: web::Data<Arc<Mutex<AppNode<V, T, S>>>>,
    call: web::Query<StateQuery>,
) -> impl Responder where
V: Serialize + DeserializeOwned + std::marker::Send + Clone,
T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher,
S: StateMachine<V, T> + std::marker::Send,
{
    let app = service.lock().await;
    let deserialized_call: StateQuery = call.into_inner();
    let key: H256 = H256::from(hex_string_to_u8_array(&deserialized_call.key));
    
    let state_with_proof = match app.get_state_with_proof(&key).await {
      Ok(i) => i,
      Err(e) => return HttpResponse::InternalServerError().body("Internal error.")
    };
    
    HttpResponse::Ok().json(state_with_proof)
}

async fn api_handler<V, T, S>(
    service: web::Data<Arc<Mutex<AppNode<V, T, S>>>>,
    call: web::Json<T>,
) -> impl Responder
where
    V: Serialize + DeserializeOwned + std::marker::Send + Clone,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher,
    S: StateMachine<V, T> + std::marker::Send,
{
    let mut app = service.lock().await;
    println!("Adding transaction to pool.");

    app.add_to_tx_pool(call.clone()).await;

    "Transaction Added to batch."
}

pub async fn start_rpc_server<V, T, S>(singleton: AppNode<V, T, S>, port: u16)
where
    V: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher,
    S: StateMachine<V, T> + std::marker::Send + 'static,
{
    let shared_service = Arc::new(Mutex::new(singleton));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_service.clone()))
            .route("/", web::post().to(api_handler::<V, T, S>))
            .route("/state", web::get().to(get_state_with_proof::<V, T, S>))
    })
    .bind(("127.0.0.1", port))
    .unwrap()
    .run()
    .await;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AppChain {
    Nft,
    Payments,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitProofParam {
    session_receipt: SessionReceipt,
    receipts: Vec<TransactionReceipt>,
    chain: AppChain,
}
