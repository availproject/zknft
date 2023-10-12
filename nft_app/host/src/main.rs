use nft_core::{
    nft::{
        state_machine::NftStateMachine,
        types::{Nft, NftTransaction},
    },
    traits::StateMachine,
    app_node::{AppNode, AppNodeConfig, RPCServer, api_handler}, 
    types::{AppChain, RPCMethod}
};
use nft_methods::{TRANSFER_ELF, TRANSFER_ID};





use std::sync::Arc;
use tokio::sync::Mutex;

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

        RPCServer::new(Arc::new(Mutex::new(app_clone)), 7000).run(
            vec![
                RPCMethod::new(String::from("/"), api_handler::<Nft, NftTransaction, NftStateMachine>)
            ]
        ).await;
    });
    
    
}
