mod rpc_endpoints;
mod types;
use nft_core::{
    app_node::{AppNode, AppNodeConfig, RPCServer},
    payments::{
        state_machine::PaymentsStateMachine,
        types::{Account, CallType, Transaction},
    },
    traits::StateMachine,
    types::{AppChain, ClientReply},
};
use payments_methods::{TRANSFER_ELF, TRANSFER_ID};
use primitive_types::U256;
use risc0_zkvm::{
    serde::{from_slice, to_vec},
    ExecutorEnv,
};
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::{
    default_store::DefaultStore, error::Error, traits::Hasher, traits::Value, MerkleProof,
    SparseMerkleTree, H256,
};
use std::time::SystemTime;
use tokio::sync::Mutex;
use std::sync::Arc;
use warp::Filter;
use warp::Rejection;
use warp::http::StatusCode;
use warp::Reply;
use crate::rpc_endpoints::routes;
use std::convert::Infallible;

fn main() {
    println!("Starting Payments app chain with zkvm id: {:?}", &TRANSFER_ID);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(async move {AppNode::<Account, Transaction, PaymentsStateMachine>::new(
        AppNodeConfig { 
            prover_mode: true, 
            light_client_url: String::from("http://127.0.0.1:8001"), 
            node_client_url: String::from("wss://kate.avail.tools:443/ws"),
            seed: String::from("clock network cage hen enough climb pencil visual spike eye marriage globe"),
            app_id: 8, 
        },
        TRANSFER_ELF,
        TRANSFER_ID,
        AppChain::Payments,
    ).await });

    let app_clone = app.clone();
    rt.block_on(async move {
        tokio::spawn(async move { app.run().await });

        let mutex_app = Arc::new(Mutex::new(app_clone.clone()));
        let nft_routes = routes(mutex_app.clone());
        let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["content-type"]);

        let routes = nft_routes.with(cors).recover(handle_rejection);
        RPCServer::new(mutex_app, String::from("127.0.0.1"), 7001).run(routes).await;
    });
    
    ()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorMessage {
    code: u16, 
    message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let mut code = StatusCode::OK;
    let mut message = "OK";

    println!("{:?}", &err);

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(ClientReply) = err.find::<ClientReply<String>>() {
        code = StatusCode::BAD_REQUEST;
        message = "BAD_REQUEST";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
