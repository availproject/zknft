use crate::db::NodeDB;
use crate::errors::Error;
use crate::state::VmState;
use crate::traits::StateMachine;
use crate::traits::TxHasher;
use crate::types::AggregatedBatch;
use crate::types::Batch;
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

//Below imports for HTTP server.
use actix_web::error;
use actix_web::rt::System;
use actix_web::HttpResponse;
use actix_web::{get, web, App, HttpServer, Responder};
use reqwest;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

pub struct AppNodeRuntimeConfig {
    pub prover_mode: bool,
}

pub struct AppNode<V, T: Clone + DeserializeOwned + Serialize, S: StateMachine<V, T>> {
    state_machine: S,
    db: NodeDB,
    runtime_config: AppNodeRuntimeConfig,
    chain: AppChain,
    zkvm_elf: Box<[u8]>,
    zkvm_id: Digest,
    phantom_v: PhantomData<V>,
    tx_pool: Vec<T>,
}

impl<
        V: Serialize + DeserializeOwned,
        T: Clone + DeserializeOwned + Serialize + TxHasher,
        S: StateMachine<V, T>,
    > AppNode<V, T, S>
{
    pub fn new(
        config: AppNodeRuntimeConfig,
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
        let state_machine = S::new(last_state_root);

        Self {
            state_machine,
            db: node_db,
            runtime_config: config,
            chain,
            zkvm_elf: zkvm_elf.into(),
            zkvm_id: zkvm_id.into(),
            phantom_v: PhantomData,
            tx_pool: vec![]
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }

    //TODO: Complete implementation.
    pub async fn sync(&mut self) -> Result<(), Error> {
        let start_height = 283562;
        let light_client_url = "http://127.0.0.1:8000".to_string();
        // Initialize the Avail service using the DaService interface
        let da_service = AvailDaProvider::new(DaServiceConfig {
            node_client_url: "wss://kate.avail.tools:443/ws".to_string(),
            light_client_url,
            seed: String::from("demo_seed"), 
            app_id: 7,
        }).await;

        for height in start_height.. {
            let filtered_block = match da_service.get_finalized_at(height).await {
                Ok(i) => i,
                Err(e) => panic!("{}", e.to_string()),
            };

            let batch: Option<Batch<T>> = match filtered_block.transactions.is_empty() {
                true => None,
                false => from_json_slice(&filtered_block.transactions[0].blob()).unwrap(),
            };

            match batch {
                None => println!("no batches in block"),
                Some(i) => println!("Found batch."),
            }
        }

        Ok(())
    }

    pub async fn execute_batch(&mut self, call_params: T) -> Result<(), Error> {
        let nexus_url = "http://127.0.0.1:8080/current-batch";
        let now = SystemTime::now();
        let last_batch_number: u64 = match &self.db.get::<BatchHeader>(b"last_batch_header") {
            Ok(Some(i)) => i.batch_number,
            Ok(None) => 0,
            Err(e) => panic!("Could not start node. {:?}", e),
        };
        //TODO: Add proper error handling below by removing unwrap and store last
        //batch in memory.
        let aggregated_proof: AggregatedBatch =
            reqwest::get(nexus_url).await.unwrap().json().await.unwrap();

        //TODO: Below should be replaced with a loop to execute a list of transactions.
        let (state_update, receipt) = self
            .state_machine
            .execute_tx(call_params.clone(), aggregated_proof.clone())
            .unwrap();

        // println!(
        //     "Pre state: {:?}, Post state: {:?}",
        //     &state_update.pre_state_root, &state_update.post_state_root
        // );
        let env = ExecutorEnv::builder()
            .add_input(&to_vec(&call_params).unwrap())
            .add_input(&to_vec(&state_update).unwrap())
            .add_input(&to_vec(&(last_batch_number + 1)).unwrap())
            .add_input(&to_vec(&aggregated_proof).unwrap())
            .build()
            .unwrap();

        // Next, we make an executor, loading the (renamed) ELF binary.
        let mut exec = default_executor_from_elf(env, &self.zkvm_elf).unwrap();
        // Run the executor to produce a session.
        let session = exec.run().unwrap();
        let segments = session.resolve().unwrap();

        let cycles = segments
            .iter()
            .fold(0, |acc, segment| acc + (1 << segment.po2));

        println!("Executed, cycles: {}k", cycles / 1024);
        // Prove the session to produce a receipt.
        let session_receipt = session.prove().unwrap();

        println!("Session executed in zkvm with ID {:?}", &self.zkvm_id);
        session_receipt.verify(self.zkvm_id).unwrap();

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
        let client = reqwest::Client::new();
        let url = "http://localhost:8080/submit-batch"; // Change this to your server's URL

        let serialized = serde_json::to_string(&SubmitProofParam {
            session_receipt,
            receipts: vec![receipt.clone()],
            chain: self.chain.clone(),
        })
        .unwrap();

        let response = client
            .post(url)
            .body(serialized)
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap();

        let status = response.status();

        println!("Batch submit call resovled with status: {:?}", status);

        //Add batch header.
        match &self.db.put(
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

        //TODO: Add tx list. For now only one transaction.
        match &self.db.put(
            call_params.to_h256().as_slice(),
            &TransactionWithReceipt {
                transaction: call_params,
                receipt: receipt,
            },
        ) {
            Ok(()) => (),
            Err(e) => return Err(e.clone()),
        }

        Ok(())
    }

    pub fn get_state_with_proof(&self, key: &H256) -> Result<(V, MerkleProof), Error>{
        self.state_machine.get_state_with_proof(key)
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
V: Serialize + DeserializeOwned + std::marker::Send,
T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher,
S: StateMachine<V, T> + std::marker::Send,
{
    let app = service.lock().unwrap();
    let deserialized_call: StateQuery = call.into_inner();
    let key: H256 = H256::from(hex_string_to_u8_array(&deserialized_call.key));
    
    let state_with_proof = match app.get_state_with_proof(&key) {
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
    V: Serialize + DeserializeOwned + std::marker::Send,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher,
    S: StateMachine<V, T> + std::marker::Send,
{
    let mut app = service.lock().unwrap();
    println!("Received request for transaction. Starting batch execution.");

    app.execute_batch(call.clone()).await;

    "Transaction Executed"
}

pub async fn start_rpc_server<V, T, S>(singleton: AppNode<V, T, S>, port: u16)
where
    V: Serialize + DeserializeOwned + std::marker::Send + 'static,
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
