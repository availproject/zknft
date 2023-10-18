mod rpc_endpoints;
use nft_core::{
    nft::{
        state_machine::NftStateMachine,
        types::{Nft, NftTransaction},
    },
    traits::StateMachine,
    app_node::{AppNode, AppNodeConfig, RPCServer, routes},
    types::{AppChain}
};
use nft_methods::{TRANSFER_ELF, TRANSFER_ID};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::rpc_endpoints::nft_routes;
use warp::Filter;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(async move {AppNode::<Nft, NftTransaction, NftStateMachine>::new(
        AppNodeConfig {
            prover_mode: true, 
            light_client_url: String::from("http://127.0.0.1:8001"), 
            node_client_url: String::from("wss://kate.avail.tools:443/ws"),
            seed: String::from("clock network cage hen enough climb pencil visual spike eye marriage globe"),
            app_id: 7,
        }, 
        TRANSFER_ELF, 
        TRANSFER_ID, 
        AppChain::Nft
    ).await });

    let app_clone = app.clone();
    rt.block_on(async move {
        tokio::spawn(async move { app.run().await });

        let mutex_app = Arc::new(Mutex::new(app_clone.clone()));
        let routes = routes(mutex_app.clone()).or(nft_routes(mutex_app.clone()));
        RPCServer::new(mutex_app, String::from("127.0.0.1"), 7000).run(routes).await;
    });
}
