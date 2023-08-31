mod errors;
mod nexus_app;
mod store;

use nexus_app::{start_rpc_server, NexusApp};
use nft_core::{
    db::NodeDB,
    state::VmState,
    types::{BatchHeader},
};
use crate::nexus_app::{AppState, AggregatedBatch};
use nft_methods::TRANSFER_ID as NFT_ID;
use payments_methods::TRANSFER_ID as TRANSFER_ID;
use std::sync::{Arc, Mutex};
use sparse_merkle_tree::H256;
use std::thread;

#[tokio::main]
async fn main() {
    println!("Nexus started, ZKVM IDs: {:?}, and {:?}",NFT_ID, TRANSFER_ID);
    let db = NodeDB::from_path(String::from("./nexus_db"));
    let last_aggregated_batch: AggregatedBatch = match db.get::<AggregatedBatch>(b"last_aggregated_proof") {
        Ok(Some(i)) => i.clone(),
        Ok(None) => AggregatedBatch {
            proof_number: 0, 
            receipts_root: H256::zero()
        },
        Err(e) => panic!("Could not start node. {:?}", e),
    };
    let last_aggregated_nft_batch: BatchHeader = match db.get::<BatchHeader>(b"last_aggregated_nft_batch") {
        Ok(Some(i)) => i.clone(),
        Ok(None) => BatchHeader::default(),
        Err(e) => panic!("Could not start node. {:?}", e),
    };
    let last_aggregated_payments_batch: BatchHeader = match db.get::<BatchHeader>(b"last_aggregated_payments_batch") {
        Ok(Some(i)) => i.clone(),
        Ok(None) => BatchHeader::default(),
        Err(e) => panic!("Could not start node. {:?}", e),
    };

    let shared_tree = Arc::new(Mutex::new(VmState::new(last_aggregated_batch.receipts_root.clone())));
    let shared_db = Arc::new(Mutex::new(db));
    let shared_app_state =  Arc::new(Mutex::new(AppState::new(
        last_aggregated_batch, 
        last_aggregated_nft_batch, 
        last_aggregated_payments_batch
    )));

    let mut app = NexusApp::new(
        shared_tree, 
        shared_app_state, 
        shared_db
    );
    let app_clone = app.clone();
    // let shared_service = Arc::new(Mutex::new(app.clone()));
    // let shared_service_clone = shared_service.clone();
    // Spawn a new thread for the RPC server
    let rpc_thread = thread::spawn(move || {
        start_rpc_server(app_clone);
    });

    // Start the main loop in the current thread
    //let mut app_lock = shared_service.lock().unwrap();
    app.start().await;

    // Wait for the RPC server thread to finish
    rpc_thread.join().unwrap();
}
