use crate::errors::{AppError, Error, ProofError};
use nft_core::{
    db::NodeDB,
    state::VmState,
    types::{BatchHeader, TransactionReceipt},
};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::H256;

//Below imports for HTTP server.
use actix_web::error;
use actix_web::rt::System;
use actix_web::HttpResponse;
use actix_web::{get, web, App, HttpServer, Responder};
use nft_methods::TRANSFER_ID as NFT_ID;
use payments_methods::TRANSFER_ID as PAYMENTS_ID;
use risc0_zkvm::{
  SessionReceipt, 
  serde::{from_slice}
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct NexusApp {
    tree_state: Arc<Mutex<VmState<TransactionReceipt>>>,
    app_state: Arc<Mutex<AppState>>,
    db: Arc<Mutex<NodeDB>>,
}

pub struct AppState {
    pub last_aggregated_batch: AggregatedBatch,
    pub last_aggregated_nft_batch: BatchHeader,
    pub last_aggregated_payments_batch: BatchHeader,
    pub verified_nft_batches: OrderedBatches,
    pub verified_payments_batches: OrderedBatches,
}

impl AppState {
    pub fn new(
        last_aggregated_batch: AggregatedBatch,
        last_aggregated_nft_batch: BatchHeader,
        last_aggregated_payments_batch: BatchHeader,
    ) -> Self {
        AppState {
            last_aggregated_batch,
            last_aggregated_nft_batch,
            last_aggregated_payments_batch,
            verified_nft_batches: OrderedBatches::new(),
            verified_payments_batches: OrderedBatches::new(),
        }
    }

    pub fn get_last_nft_verified_batch(&self) -> BatchHeader {
      if self.verified_nft_batches.proof_count() == 0 {
        self.last_aggregated_nft_batch.clone()
      } else {
        match self.verified_nft_batches.last() {
          Some(i) => i.header.clone(), 
          None => unreachable!("Will not be empty in else.")
        }
      }
    }

    pub fn get_last_payments_verified_batch(&self) -> BatchHeader {
      if self.verified_payments_batches.proof_count() == 0 {
        self.last_aggregated_payments_batch.clone()
      } else {
        match self.verified_payments_batches.last() {
          Some(i) => i.header.clone(), 
          None => unreachable!("Will not be empty in else.")
        }
      }
    }
}

pub struct BatchWithReceipts {
    receipts: Vec<TransactionReceipt>,
    header: BatchHeader,
}

pub struct OrderedBatches(Vec<BatchWithReceipts>);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct AggregatedBatch {
    pub proof_number: u64,
    pub receipts_root: H256,
}

impl OrderedBatches {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn last(&self) -> Option<&BatchWithReceipts> {
        self.0.last()
    }

    pub fn first(&self) -> Option<&BatchWithReceipts> {
        self.0.first()
    }

    pub fn add_batch(&mut self, batch: BatchWithReceipts) -> () {
        self.0.push(batch);
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
    pub fn new(
      tree_state: Arc<Mutex<VmState<TransactionReceipt>>>,
      app_state: Arc<Mutex<AppState>>,
      db: Arc<Mutex<NodeDB>>, 
    ) -> Self {
      Self {
        tree_state,
        app_state,
        db
      }
    }

    pub async fn start(&self) -> () {

        // loop  {
        //   println!("Task executed! {}", self.last_proof.lock().unwrap());

        //   tokio::time::sleep( Duration::from_secs(5)).await;
        // }
    }

    // pub fn update_proof(&mut self, proof: &str) -> () {
    //     let mut proof_store = self.last_proof.lock().unwrap();

    //     proof_store.clear();

    //     proof_store.push_str(proof);
    // }

    pub fn verify_nft_batch(&mut self, receipt: SessionReceipt, tx_receipts: Vec<TransactionReceipt>) -> Result<(), Error> {
        let mut app_state = self.app_state.lock().unwrap();

        match receipt.verify(NFT_ID) {
            Ok(i) => Ok::<(), ProofError>(()),
            //TODO: Simplify this chaining.
            Err(e) => return Err(Error::ProofError(ProofError(String::from(
                "Unable to verify proof",
            )))),
        };

        let batch_header: BatchHeader = from_slice(&receipt.journal).unwrap();
        let last_batch_header: BatchHeader = app_state.get_last_nft_verified_batch();
        //TODO: change this to calculate root of all receipts, currently we assume
        //there is only one receipt per batch.
        let receipts_root: H256 = tx_receipts[0].to_h256();

        //TODO: Seperate the check for better error response.
        if receipts_root == batch_header.receipts_root &&
        last_batch_header.state_root == batch_header.pre_state_root {
          app_state.verified_nft_batches.add_batch(
            BatchWithReceipts {
              header: batch_header, 
              receipts: tx_receipts
            }
          );

          Ok(())
        } else {
          Err(Error::ProofError(ProofError(String::from(
            "Invalid proof",
        ))))
        }
    }

    pub fn verify_payments_batch(&mut self, receipt: SessionReceipt, tx_receipts: Vec<TransactionReceipt>) -> Result<(), Error> {
      let mut app_state = self.app_state.lock().unwrap();

      match receipt.verify(PAYMENTS_ID) {
          Ok(i) => Ok::<(), ProofError>(()),
          //TODO: Simplify this chaining.
          Err(e) => return Err(Error::ProofError(ProofError(String::from(
              "Unable to verify proof",
          )))),
      };

      let batch_header: BatchHeader = from_slice(&receipt.journal).unwrap();
      let last_batch_header: BatchHeader = app_state.get_last_payments_verified_batch();
      //TODO: change this to calculate root of all receipts, currently we assume
      //there is only one receipt per batch.
      let receipts_root: H256 = tx_receipts[0].to_h256();

      //TODO: Seperate the check for better error response.
      if receipts_root == batch_header.receipts_root &&
      last_batch_header.state_root == batch_header.pre_state_root {
        app_state.verified_payments_batches.add_batch(
          BatchWithReceipts {
            header: batch_header, 
            receipts: tx_receipts
          }
        );

        Ok(())
      } else {
        Err(Error::ProofError(ProofError(String::from(
          "Invalid proof",
      ))))
      }
  }
}


#[derive(Debug, Deserialize, Serialize)]
pub enum AppChain {
  Nft, 
  Payments,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitProofParam {
  session_receipt: SessionReceipt, 
  receipts: Vec<TransactionReceipt>, 
  chain: AppChain
}

async fn submit_batch(
    service: web::Data<Arc<Mutex<NexusApp>>>,
    call: web::Json<SubmitProofParam>,
) -> impl Responder {
    let deserialized_call: SubmitProofParam = call.into_inner();
    let mut app = service.lock().unwrap();

    match deserialized_call.chain {
      AppChain::Nft => match app.verify_nft_batch(deserialized_call.session_receipt, deserialized_call.receipts) {
        Ok(()) => "Proof Submitted", 
        Err(e) => "Proof not submitted", 
      }, 
      AppChain::Payments => match app.verify_payments_batch(deserialized_call.session_receipt, deserialized_call.receipts) {
        Ok(()) => "Proof Submitted", 
        Err(e) => "Proof not submitted", 
      },
    }
}

pub async fn start_rpc_server(shared_service: Arc<Mutex<NexusApp>>) -> impl Send {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_service.clone()))
            .route("/submit-batch", web::post().to(submit_batch))
    })
    .bind(("127.0.0.1", 8000))
    .unwrap()
    .run()
    .await;
}