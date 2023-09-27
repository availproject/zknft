mod errors;
mod nexus_app;
mod store;

use crate::nexus_app::{AggregatedBatch, AppState};
use nexus_app::{start_rpc_server, NexusApp};
use nft_core::{db::NodeDB, state::VmState, types::BatchHeader};
use nft_methods::TRANSFER_ID as NFT_ID;
use avail::service::{DaProvider, DaServiceConfig};
use payments_methods::TRANSFER_ID;
use sparse_merkle_tree::H256;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::time::Duration;

fn main() {
    println!(
        "Nexus started, ZKVM IDs: {:?}, and {:?}",
        NFT_ID, TRANSFER_ID
    );
    let db = NodeDB::from_path(String::from("./nexus_db"));
    let last_aggregated_batch: AggregatedBatch =
        match db.get::<AggregatedBatch>(b"last_aggregated_proof") {
            Ok(Some(i)) => i.clone(),
            Ok(None) => AggregatedBatch {
                proof_number: 0,
                receipts_root: H256::zero(),
            },
            Err(e) => panic!("Could not start node. {:?}", e),
        };
    let last_aggregated_nft_batch: BatchHeader =
        match db.get::<BatchHeader>(b"last_aggregated_nft_batch") {
            Ok(Some(i)) => i.clone(),
            Ok(None) => BatchHeader::default(),
            Err(e) => panic!("Could not start node. {:?}", e),
        };
    let last_aggregated_payments_batch: BatchHeader =
        match db.get::<BatchHeader>(b"last_aggregated_payments_batch") {
            Ok(Some(i)) => i.clone(),
            Ok(None) => BatchHeader::default(),
            Err(e) => panic!("Could not start node. {:?}", e),
        };

    let shared_tree = Arc::new(Mutex::new(VmState::new(
        last_aggregated_batch.receipts_root.clone(),
    )));
    let shared_db = Arc::new(Mutex::new(db));
    let shared_app_state = Arc::new(Mutex::new(AppState::new(
        last_aggregated_batch,
        last_aggregated_nft_batch,
        last_aggregated_payments_batch,
    )));
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let nft_da_service = rt.block_on(async move {DaProvider::new(DaServiceConfig {
        light_client_url: String::from("http://127.0.0.1:8000"), 
        node_client_url: String::from("wss://kate.avail.tools:443/ws"),
        seed: String::from("rose label choose orphan garlic upset scout payment first have boil stamp"), 
        app_id: 7,
    }).await});
    let payments_da_service = rt.block_on(async move {DaProvider::new(DaServiceConfig {
        light_client_url: String::from("http://127.0.0.1:8001"), 
        node_client_url: String::from("wss://kate.avail.tools:443/ws"),
        seed: String::from("rose label choose orphan garlic upset scout payment first have boil stamp"), 
        app_id: 8,
    }).await});

    let mut app = NexusApp::new(
        shared_tree, 
        shared_app_state, 
        shared_db, 
        465660,
        nft_da_service, 
        payments_da_service,
    );
    let app_clone = app.clone();

    rt.block_on(async move {
        tokio::spawn(async move { app.start().await });

        start_rpc_server(app_clone).await;
    })
}
