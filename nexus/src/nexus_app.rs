use nft_core::{db::NodeDB, state::VmState, types::BatchHeader};
use std::thread;
use std::time::Duration;

//Below imports for HTTP server.
use actix_web::error;
use actix_web::rt::System;
use actix_web::HttpResponse;
use actix_web::{get, web, App, HttpServer, Responder};
use std::sync::{Arc, Mutex};
use nft_methods::{TRANSFER_ID};
use risc0_zkvm::{
  SessionReceipt
};

#[derive(Clone)]
pub struct NexusApp {
  pub state: VmState<Vec<u8>>,
  pub last_aggregated_nft_batch: Arc<Mutex<BatchHeader>>,
  pub last_aggregated_payments_batch: Arc<Mutex<BatchHeader>>,
  pub verified_nft_batch: Arc<Mutex<>>
}

pub struct BatchWithReceipts<Vec<u8>> {
  receipts: Vec<V>, 
  header: BatchHeader
}

pub struct OrderedProofs(Vec<BatchHeader>);

impl OrderedProofs {
  pub fn last(&self) -> Option<&BatchHeader> {
    self.0.last()
  }

  pub fn first(&self) -> Option<&BatchHeader> {
    self.0.first()
  }

  pub fn add_proof(&mut self, header: BatchHeader) -> () {
    self.0.push(header);
  }

  pub fn delete_first(&mut self) -> () {
    if !self.0.is_empty() {
      self.0.remove(0);
    } 
  }

  pub fn proof_count(&self) -> usize {
    self.0.len()
  }
}

impl NexusApp {
  pub async fn start(&self) -> () {

    // loop  {
    //   println!("Task executed! {}", self.last_proof.lock().unwrap());
      
    //   tokio::time::sleep( Duration::from_secs(5)).await;
    // }
  }

  pub fn update_proof(&mut self, proof: &str) -> () {
    let mut proof_store = self.last_proof.lock().unwrap();

    proof_store.clear();

    proof_store.push_str(proof);
  }
}

async fn api_handler(
  service: web::Data<Arc<Mutex<NexusApp>>>,
  call: web::Json<SessionReceipt>,
) -> impl Responder
{
  let deserialized: SessionReceipt = call.into_inner();
  deserialized.verify(TRANSFER_ID).unwrap();

  "Transaction Executed"
}

pub async fn start_rpc_server(shared_service: Arc<Mutex<NexusApp>>)
-> impl Send {
  HttpServer::new(move || {
      App::new()
          .app_data(web::Data::new(shared_service.clone()))
          .route("/", web::post().to(api_handler))
  })
  .bind(("127.0.0.1", 8000))
  .unwrap()
  .run()
  .await;
}
