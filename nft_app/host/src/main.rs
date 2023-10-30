mod rpc_endpoints;
mod types;
use nft_core::{
    nft::{
        state_machine::NftStateMachine,
        types::{Nft, NftTransaction},
    },
    traits::StateMachine,
    app_node::{AppNode, AppNodeConfig, RPCServer, routes},
    types::{AppChain, Address, ClientReply}
};
use nft_methods::{TRANSFER_ELF, TRANSFER_ID};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::rpc_endpoints::nft_routes;
use ed25519_consensus::{SigningKey};
use ed25519_consensus::Signature;
use warp::Filter;
use warp::Rejection;
use warp::http::StatusCode;
use warp::Reply;
use std::convert::Infallible;
use serde::{ Serialize, Deserialize};

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

    rt.block_on(async move {
        let app_clone = app.clone();
        let execution_engine = tokio::spawn(async move {
            loop {
                let execution_app = app.clone();
                let execution = tokio::spawn(async move {execution_app.run().await;});

                let result = tokio::try_join!(
                    execution,
                );
            
                match result {
                    Ok(_) => {
                        println!("Thread completed successfully.");
                    }
                    Err(e) => {
                        println!("Thread failed due to panic. restarting node. {:?}", e);
                    }
                }
            }
        });

        let mutex_app = Arc::new(Mutex::new(app_clone.clone()));
        let nft_routes = routes(mutex_app.clone()).or(nft_routes(mutex_app.clone(), signing_key));
        let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["content-type"]);

        let routes = nft_routes.with(cors);
        let rpc = tokio::spawn(async move {  RPCServer::new(mutex_app, String::from("127.0.0.1"), 7000).run(routes).await; });

        let result = tokio::try_join!(
            execution_engine,
            rpc,
        );
    
        match result {
            Ok((_, _)) => {
                println!("Exiting node, should not have happened.");
            }
            Err(e) => {
                println!("Exiting node, should not have happened.");
            }
        }
    });
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorMessage {
    code: u16, 
    message: String,
}

// async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
//     let mut code = StatusCode::OK;
//     let mut message = "OK";

//     println!("ERRR: {:?}", &err);

//     if err.is_not_found() {
//         code = StatusCode::NOT_FOUND;
//         message = "NOT_FOUND";
//     } else {
//         code = StatusCode::BAD_REQUEST;
//         message = "BAD_REQUEST";
//     }

//     let json = warp::reply::json(&ErrorMessage {
//         code: code.as_u16(),
//         message: message.into(),
//     });

//     Ok(warp::reply::with_status(json, code))
// }
