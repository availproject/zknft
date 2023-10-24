mod rpc_endpoints;
use nft_core::{
    nft::{
        state_machine::NftStateMachine,
        types::{Nft, NftTransaction},
    },
    traits::StateMachine,
    app_node::{AppNode, AppNodeConfig, RPCServer, routes},
    types::{AppChain, Address}
};
use nft_methods::{TRANSFER_ELF, TRANSFER_ID};
use std::sync::Arc;
use tokio::sync::Mutex;
use sparse_merkle_tree::H256;
use crate::rpc_endpoints::nft_routes;
use ed25519_consensus::{SigningKey};
use warp::Filter;
use serde::{ de::DeserializeOwned, Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    keypair_bytes: [u8; 32]
}

fn main() {
    println!("Starting NFT app chain with zkvm id: {:?}", &TRANSFER_ID);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(async move {AppNode::<Nft, NftTransaction, NftStateMachine>::new(
        AppNodeConfig {
            prover_mode: true, 
            light_client_url: String::from("http://127.0.0.1:8000"), 
            node_client_url: String::from("wss://kate.avail.tools:443/ws"),
            seed: String::from("clock network cage hen enough climb pencil visual spike eye marriage globe"),
            app_id: 7,
        },
        TRANSFER_ELF, 
        TRANSFER_ID, 
        AppChain::Nft
    ).await });
    let json_data = std::fs::read_to_string("keypair.json").unwrap();
    let keypair_data: Data = serde_json::from_str(&json_data).unwrap();
    // Create a SigningKey from the deserialized keypair_bytes
    let signing_key: SigningKey = SigningKey::from(keypair_data.keypair_bytes);
    let verifying_key = Address(signing_key.verification_key().to_bytes());
    
    rt.block_on(async {
        let app_clone = app.clone();
        let mut state_machine = app_clone.state_machine.lock().await;
        state_machine.register_custodian(verifying_key);
    });

    let app_clone = app.clone();
    rt.block_on(async move {
        tokio::spawn(async move { app.run().await });

        let mutex_app = Arc::new(Mutex::new(app_clone.clone()));
        let routes = routes(mutex_app.clone()).or(nft_routes(mutex_app.clone()));
        RPCServer::new(mutex_app, String::from("127.0.0.1"), 7000).run(routes).await;
    });
}
