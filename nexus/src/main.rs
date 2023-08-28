mod nexus_app;
mod store;

use nexus_app::{NexusApp, start_rpc_server};
use std::thread;
use std::sync::{Arc, Mutex};
use nft_methods::{TRANSFER_ID};

#[tokio::main]
async fn main() {
    println!("{:?}", TRANSFER_ID);
    let shared_data = Arc::new(Mutex::new(String::from("Hi")));
    let app = NexusApp {
        last_proof: shared_data,
    };
    let shared_service = Arc::new(Mutex::new(app.clone()));
    let shared_service_clone = shared_service.clone();
    // Spawn a new thread for the RPC server
    let rpc_thread = thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(start_rpc_server(shared_service_clone));
    });

    // Start the main loop in the current thread
    //let app_lock = shared_service.lock().unwrap();
    app.start().await;

    // Wait for the RPC server thread to finish
    rpc_thread.join().unwrap();

    println!("Hello, world!");
}
