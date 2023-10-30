use crate::db::NodeDB;
use crate::traits::StateMachine;
use crate::traits::TxHasher;
use crate::types::AggregatedBatch;
use crate::types::DABatch;
use crate::types::BatchHeader;
use crate::types::TransactionWithReceipt;
use crate::types::BatchWithProof;
use crate::types::ClientReply;
use crate::types::{DaTxPointer, SubmitProofParam, AppChain};
use crate::utils::hex_string_to_u8_array;
use avail::service::{DaProvider as AvailDaProvider, DaServiceConfig};
use risc0_zkp::core::digest::Digest;
use risc0_zkvm::{
    serde::{from_slice, to_vec},
    ExecutorEnv,
    Executor
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use parity_scale_codec::{Encode, Decode};
use anyhow::{Error, anyhow};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;
use sparse_merkle_tree::MerkleProof;
use std::marker::PhantomData;
use std::time::SystemTime;
use std::time::Duration;
use std::net::SocketAddr;
use std::str::FromStr;
use anyhow::Context;
use std::io::prelude::*;
use core::convert::Infallible;
//Below imports for HTTP server.
use warp::{Filter, reply::Reply, Rejection};
use reqwest;
use std::sync::Arc;
use tokio::sync::Mutex;

const NEXUS_SUBMIT_BATCH_URL: &str = "http://127.0.0.1:8080/submit-batch";
const NEXUS_LATEST_BATCH_URL: &str = "http://127.0.0.1:8080/current-batch";

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

pub struct AppNode<V: Clone + Encode + Decode, T: Clone + DeserializeOwned + Serialize + Encode + Decode, S: StateMachine<V, T>> {
    pub state_machine: Arc<Mutex<S>>,
    db: Arc<Mutex<NodeDB>>,
    da_service: AvailDaProvider,
    chain: AppChain,
    zkvm_elf: Box<[u8]>,
    zkvm_id: Digest,
    phantom_v: PhantomData<V>,
    tx_pool: Arc<Mutex<Vec<T>>>,
}

impl<
        V: Serialize + DeserializeOwned + Clone + Encode + Decode,
        T: Clone + DeserializeOwned + Serialize + TxHasher + Encode + Decode,
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
            zkvm_id: self.zkvm_id,
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
            Ok(Some(i)) => i.state_root,
            Ok(None) => H256::zero(),
            Err(e) => panic!("Could not start node. {:?}", e),
        };
        let state_machine = Arc::new(Mutex::new(S::new(last_state_root.clone())));
        let da_service = AvailDaProvider::new(DaServiceConfig {
            node_client_url: config.node_client_url,
            light_client_url: config.light_client_url,
            seed: config.seed,
            app_id: config.app_id,
        }).await;

        println!("Creating instance of node at state root: {:?}", &last_state_root);

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

    pub async fn run(&self) -> Result<(), Error> {
        {   
            //Below code is just to cleanup merkle tree and tx pool, to start from last known stable state.
            let mut tx_pool = self.tx_pool.lock().await;

            //TODO: Only clear if failure due to transaction was detected.
            tx_pool.clear();

            println!("Cleared tx pool, before starting.");

            let last_state_root: H256 = {
                let db = self.db.lock().await;

                match db.get::<BatchHeader>(b"last_batch_header") {
                Ok(Some(i)) => i.state_root,
                Ok(None) => H256::zero(),
                Err(e) => panic!("Could not start node. {:?}", e),
                }
            }; 

            let mut state_machine = self.state_machine.lock().await;

            match state_machine.revert(last_state_root) {
                Ok(()) => (),
                //TODO: Need to restart the service on this error.
                Err(e) => panic!("Reverting state failed. Need to restart node."),
            };
        }

        loop {
            {
                let mut tx_pool = self.tx_pool.lock().await;

                while !tx_pool.is_empty() {
                    let last_state_root: H256 = {
                        let db = self.db.lock().await;

                        match db.get::<BatchHeader>(b"last_batch_header") {
                        Ok(Some(i)) => i.state_root,
                        Ok(None) => H256::zero(),
                        Err(e) => panic!("Could not start node. {:?}", e),
                        }
                    }; 

                    match self.execute_batch(tx_pool[0].clone()).await {
                        Ok(()) => (), 
                        Err(e) => {
                            println!("Reverting state machine to root: {:?} due to error: {:?}", &last_state_root,e);
                            let mut state_machine = self.state_machine.lock().await;

                            match state_machine.revert(last_state_root) {
                                Ok(()) => (),
                                //TODO: Need to restart the service on this error.
                                Err(e) => panic!("Reverting state failed. Need to restart node."),
                            };
                        }
                    }

                    //TODO: Only remove if the transaction failed 
                    //due to state transition error. (or was successful)
                    tx_pool.remove(0);
                    continue;
                }
            }
            println!("Sleeping end of loop.");
            //Make this configurable.
            tokio::time::sleep(Duration::from_secs(10)).await;
        } 
    }

    pub async fn execute_batch(&self, call_params: T) -> Result<(), Error> {
        let _now = SystemTime::now();
        let last_batch_number: u64 = {
            let db = self.db.lock().await;

            match &db.get::<BatchHeader>(b"last_batch_header") {
            Ok(Some(i)) => i.batch_number,
            Ok(None) => 0,
            Err(e) => return Err(anyhow!("Could not start node. {:?}", e)),
            }
        };
        //TODO: Add proper error handling below by removing unwrap and store last
        //batch in memory.
        let response = reqwest::get(NEXUS_LATEST_BATCH_URL).await?;
        let aggregated_proof: AggregatedBatch = response.json().await?;

        let mut state_machine = self.state_machine.lock().await;

        //TODO: Below should be replaced with a loop to execute a list of transactions.
        let (state_update, receipt) = state_machine
            .execute_tx(call_params.clone(), aggregated_proof.clone())?;

        //Note: Have to do this weird construction as tokio spawn complains that 
        //env is not dropped before an async operation below so is not thread safe.
        let (batch, proof) = {
            let mut exec = {
                let env = ExecutorEnv::builder()
                    .add_input(&to_vec(&call_params)?)
                    .add_input(&to_vec(&state_update)?)
                    .add_input(&to_vec(&(last_batch_number + 1))?)
                    .add_input(&to_vec(&aggregated_proof)?)
                    .build()?;

                    Executor::from_elf(env, &self.zkvm_elf)?
            };

            // Run the executor to produce a session.
            let session = exec.run()?;
            let segments = session.resolve()?;

            let cycles = segments
                .iter()
                .fold(0, |acc, segment| acc + (1 << segment.po2));

            println!("Executed, cycles: {}k", cycles / 1024);
            let session_receipt = match session.prove() {
                Ok(i) => i, 
                Err(e) => return Err(anyhow!("{:?}", e))
            };

            println!("Session executed in zkvm with ID {:?}", &self.zkvm_id);
            session_receipt.verify(self.zkvm_id)?;
            
            //TODO: Might not need to be deserialized, and need to remove unwrap.
            let batch_header: BatchHeader = from_slice(&session_receipt.journal)?;
            let _transaction_with_receipt = TransactionWithReceipt {
                transaction: call_params.clone(),
                receipt: receipt.clone(),
            };

            (
                DABatch {
                header: batch_header,
                transactions: vec![call_params.clone()],
                }, 
                session_receipt
            )
        };

        let serialized = bincode::serialize(&batch)?;
        
        println!("Non compressed length: {},", serialized.len());

        let (block_hash, hash) = match self.da_service.send_transaction(&serialized).await {
            Ok(i) => {
                i
            }, 
            //Change from default error.
            Err(e) => {
                println!("error {:?}", e);
                return Err(anyhow!("DA data submit tx failed. {:?}", e.to_string()))
            },
        };

        let transaction_with_receipts = vec![TransactionWithReceipt {
            transaction: call_params.clone(),
            receipt: receipt.clone(),
        }];

        let serialized_receipt = match bincode::serialize(&proof) {
            Ok(i) => i, 
            Err(_e) => { return Err(anyhow!("Proof serialization failed."))}
        };

        let data = SubmitProofParam {
            session_receipt: serialized_receipt, 
            receipts: vec![receipt.clone()], 
            chain: self.chain.clone(), 
            da_tx_pointer: DaTxPointer {
                block_hash: block_hash.to_fixed_bytes(),
                hash: hash.to_fixed_bytes(),
                chain: self.chain.clone()
            },
        };

        let client = reqwest::Client::new();
        let response = match client
        .post(NEXUS_SUBMIT_BATCH_URL)
        .json(&data) // Serialize the data as JSON
        .send()
        .await {
            Ok(i) => i, 
            Err(e) => { return Err(anyhow!("Batch submission failed. {:?}", e.to_string()))}
        };

        //TODO: Need to reload previous state if call failed.
        match response.status().as_u16() {
            200 => {
                // Request was successful, handle the response here
                let response_text = response.text().await?;
                println!("Batch submission successful. Response: {}", response_text);
            }
            _ => {
                // Request failed
                println!("Batch submission failed with status code: {}", response.status());

                return Err(anyhow!("Submit batch failed, will try to execute again."));
            }
        }

        self.save_batch(BatchWithProof {
            header: batch.header, 
            transaction_with_receipts, 
            proof
        }).await
    }

    pub async fn save_batch(&self, batch_with_proof: BatchWithProof<T>) ->  Result<(), Error> {
        let db = self.db.lock().await;

        for tx in batch_with_proof.transaction_with_receipts {
            db.put(
                tx.transaction.to_h256().as_slice(),
                &tx,
            )?;
        }

        db.put(
            b"last_batch_header",
            &batch_with_proof.header,
        )?;

        db.put(
            &batch_with_proof.header.batch_number.to_be_bytes(),
            &batch_with_proof.header,
        )?;

        Ok(())
    }

    pub async fn add_to_tx_pool(&self, tx: T) {
        println!("Adding tx hash to pool: {:?}", tx.to_h256());
        let mut tx_pool = self.tx_pool.lock().await;

        tx_pool.push(tx)
    }

    pub async fn get_tx_status(&self, hash: H256) -> Result<String, Error> {
        let mut tx_pool = self.tx_pool.lock().await;

        for tx in tx_pool.iter() {
            // Check if the hash of the current item matches the target hash
            if tx.to_h256() == hash {
                return Ok(String::from("tx_pool"));
            }
        }

        let db = self.db.lock().await;
        
        match db.get::<T>(hash.as_slice()) {
            Ok(Some(_i)) => return  Ok(String::from("finalized.")),
            Ok(None) => return  Ok(String::from("dropped")),
            Err(e) => return  Err(anyhow!("{:?}", e)),
        }
    }

    pub async fn get_state_with_proof(&self, key: &H256) -> Result<(V, MerkleProof), Error> {
        let state_machine = self.state_machine.lock().await;

        state_machine.get_state_with_proof(key)
    }

    pub async fn get_root(&self) -> Result<H256, Error> {
        let state_machine = self.state_machine.lock().await;

        state_machine.get_root()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct StateQuery {
  key: String,
}

pub async fn get_state_with_proof<V, T, S>(
    service: Arc<Mutex<AppNode<V, T, S>>>,
    query: String,
) -> Result<ClientReply<(V, MerkleProof)>, Infallible> 
where
    V: Serialize + DeserializeOwned + std::marker::Send + Clone + std::marker::Sync + Encode + Decode,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher + Encode + Decode,
    S: StateMachine<V, T> + std::marker::Send,
{
    let app = service.lock().await;
    let key: H256 = H256::from(match hex_string_to_u8_array(&query) {
        Ok(i) => i,
        Err(e) => return Ok(ClientReply::Error(e))
    });
    println!("key: {:?} {:?}", &query, &key);
    
    let state_with_proof = match app.get_state_with_proof(&key).await {
      Ok(i) => i,
      Err(e) => return Ok(ClientReply::Error(e))
    };
    
    Ok(ClientReply::Ok(state_with_proof))
}

pub async fn api_handler<V, T, S>(
    service: Arc<Mutex<AppNode<V, T, S>>>,
    call: T,
) ->  Result<ClientReply<String>, Infallible>
where
    V: Serialize + DeserializeOwned + std::marker::Send + Clone + std::marker::Sync + Encode + Decode,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher + Encode + Decode,
    S: StateMachine<V, T> + std::marker::Send,
{   
    let app = service.lock().await;
    println!("Adding transaction to pool.");

    app.add_to_tx_pool(call).await;

    Ok(ClientReply::Ok(String::from("Transaction added to batch.")))
}

pub async fn get_tx_status<V, T, S> (
    service: Arc<Mutex<AppNode<V, T, S>>>,
    call: H256,
) -> Result<ClientReply<String>, Infallible>
where
    V: Serialize + DeserializeOwned + std::marker::Send + Clone + std::marker::Sync + Encode + Decode,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher + Encode + Decode,
    S: StateMachine<V, T> + std::marker::Send,
{
    let app = service.lock().await;

    match app.get_tx_status(call).await {
        Ok(i) => Ok(ClientReply::Ok(i)), 
        Err(e) => Ok(ClientReply::Error(e)),
    }
}

pub fn routes<V, T, S>(service: Arc<Mutex<AppNode<V, T, S>>>) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone 
where
    V: Serialize + DeserializeOwned + std::marker::Send + Clone + std::marker::Sync + Encode + Decode,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher + Encode + Decode,
    S: StateMachine<V, T> + std::marker::Send,
{
    let send_tx_app = service.clone();
    let tx_status_app = service.clone();
    let state_app = service.clone();

    let send_tx = warp::path!("tx")
            .and(warp::any().map(move || send_tx_app.clone()))
            .and(warp::body::json())
            .and_then(api_handler::<V, T, S>);

    let tx_status = warp::path!("tx_status")
            .and(warp::any().map(move || tx_status_app.clone()))
            .and(warp::body::json())
            .and_then(get_tx_status::<V, T, S>);

    let state_with_proof = warp::path("state")
            .and(warp::any().map(move || state_app.clone()))
            .and(warp::path::param::<String>())
            .and_then(get_state_with_proof::<V, T, S>);

    send_tx.or(tx_status).or(state_with_proof)
}

pub struct RPCServer<V, T, S> where 
V: Serialize + DeserializeOwned + std::marker::Send + Clone + Encode + Decode,
T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher + Encode + Decode,
S: StateMachine<V, T> + std::marker::Send,
{
    shared_app_node: Arc<Mutex<AppNode<V, T, S>>>, 
    port: u16,
    host: String,
}

impl<
    V: Serialize + DeserializeOwned + std::marker::Send + Clone + 'static + Encode + Decode,
    T: Serialize + DeserializeOwned + std::marker::Send + 'static + Clone + TxHasher + Encode + Decode,
    S: StateMachine<V, T> + std::marker::Send + 'static,
> RPCServer <V, T, S> 
{
    pub fn new(shared_app_node: Arc<Mutex<AppNode<V, T, S>>>, host: String, port: u16) -> Self {   

        RPCServer {
            shared_app_node, 
            host,
            port
        }
    }
    
    pub async fn run<F>(&self, routes: F)
    where 
        F: Filter + Clone + Send + Sync + 'static,
        F::Extract: Reply,
        F::Error: Into<Rejection>,
    {   
        //TODO: Maybe return error instead of panicking here.
        let address = SocketAddr::from_str(format!("{}:{}", &self.host, &self.port).as_str())
        .context("Unable to parse host address from config")
        .unwrap();

        println!("RPC Server running on: {:?}", &address);
        warp::serve(routes).run(address).await;
    }
}
