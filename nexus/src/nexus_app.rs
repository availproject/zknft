use hex;
use nft_core::traits::Leaf;
use nft_core::{
    db::NodeDB,
    state::VmState,
    types::{BatchHeader, TransactionReceipt, BatchWithProof, DABatch},
    nft::types::NftTransaction, 
    payments::types::Transaction as PaymentsTransaction,
};
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;
use std::thread;
use std::time::Duration;
use serde_json::{from_slice as from_json_slice, to_vec as to_json_vec};
use crate::types::{AppChain, DaTxPointer, SubmitProofParam, ReceiptQuery};

//Below imports for HTTP server.
use actix_web::error;
use actix_web::rt::System;
use actix_web::HttpResponse;
use actix_web::{get, web, App, HttpServer, Responder};
use avail::avail::AvailBlock;
use avail::service::DaProvider;
use avail::avail::AvailBlobTransaction;
use nft_methods::TRANSFER_ID as NFT_ID;
use payments_methods::TRANSFER_ID as PAYMENTS_ID;
use risc0_zkvm::{serde::from_slice, Receipt};
use std::sync::{Arc, Mutex};
use anyhow::Error;
use anyhow::anyhow;

const AGGREGATE_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct NexusApp {
    tree_state: Arc<Mutex<VmState<TransactionReceipt>>>,
    app_state: Arc<Mutex<AppState>>,
    db: Arc<Mutex<NodeDB>>,
    da_start_height: u64,
    nft_da_service: DaProvider,
    payments_da_service: DaProvider,
}

pub struct AppState {
    pub last_aggregated_batch: AggregatedBatch,
    pub last_aggregated_nft_batch: BatchHeader,
    pub last_aggregated_payments_batch: BatchHeader,
    pub verified_nft_batches: Vec<BatchWithReceipts>,
    pub verified_payments_batches: Vec<BatchWithReceipts>,
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
            verified_nft_batches: vec![],
            verified_payments_batches: vec![],
        }
    }

    pub fn get_last_nft_verified_batch(&self) -> BatchHeader {
        if self.verified_nft_batches.len() == 0 {
            self.last_aggregated_nft_batch.clone()
        } else {
            match self.verified_nft_batches.last() {
                Some(i) => i.header.clone(),
                None => unreachable!("Will not be empty in else."),
            }
        }
    }

    pub fn get_last_payments_verified_batch(&self) -> BatchHeader {
        if self.verified_payments_batches.len() == 0 {
            self.last_aggregated_payments_batch.clone()
        } else {
            match self.verified_payments_batches.last() {
                Some(i) => i.header.clone(),
                None => unreachable!("Will not be empty in else."),
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

    pub fn clear(&mut self) -> () {
        if !self.0.is_empty() {
            self.0.clear();
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
        da_start_height: u64,
        nft_da_service: DaProvider,
        payments_da_service: DaProvider,
    ) -> Self {
        Self {
            tree_state,
            app_state,
            db,
            da_start_height,
            nft_da_service,
            payments_da_service,
        }
    }

    pub async fn start(&mut self) -> () {
        loop {
            self.aggregate_proofs();

            tokio::time::sleep(AGGREGATE_INTERVAL).await;
        }
    }

    fn aggregate_proofs(
        &mut self,
    ) -> () {
        let mut app_state = self.app_state.lock().unwrap();
        let mut tree_state = self.tree_state.lock().unwrap();
        let mut db = self.db.lock().unwrap();
        println!(
            "aggregating proofs: {:?}, {:?}",
            app_state.verified_payments_batches.len(), app_state.verified_payments_batches.len()
        );
        let mut receipts_to_add: Vec<TransactionReceipt> = vec![];

        for batch in &app_state.verified_nft_batches {
            receipts_to_add.extend(batch.receipts.clone());
        }

        for batch in &app_state.verified_payments_batches {
            receipts_to_add.extend(batch.receipts.clone());
        }

        if receipts_to_add.len() > 0 {
            let state_update = match tree_state.update_set(receipts_to_add) {
                Ok(i) => i,
                Err(e) => {
                    println!("Panic shutdown due to error, {:?}", e);

                    panic!("State update failed.");
                }
            };

            println!(
                "New proof aggregated. root is: {:?}",
                &state_update.post_state_root
            );

            let last_aggregated_batch = AggregatedBatch {
                proof_number: app_state.last_aggregated_batch.proof_number + 1,
                receipts_root: state_update.post_state_root,
            };
            //TODO: Set this through a method.
            app_state.last_aggregated_batch = last_aggregated_batch.clone();

            match db.put::<AggregatedBatch>(b"last_aggregated_proof", &last_aggregated_batch) {
                Ok(()) => (),
                Err(e) => panic!("Could not start node. {:?}", e),
            }
            match db.put::<BatchHeader>(
                b"last_aggregated_nft_batch",
                &app_state.last_aggregated_nft_batch,
            ) {
                Ok(()) => (),
                Err(e) => panic!("Could not start node. {:?}", e),
            }
            match db.put::<BatchHeader>(
                b"last_aggregated_payments_batch",
                &app_state.last_aggregated_payments_batch,
            ) {
                Ok(()) => (),
                Err(e) => panic!("Could not start node. {:?}", e),
            }
        }

        app_state.verified_nft_batches.clear();
        app_state.verified_payments_batches.clear();
    }

    async fn get_da_tx(
        &self, 
        pointer: DaTxPointer,
    ) -> Result<AvailBlobTransaction, Error> {
        let da_service = match pointer.chain {
            AppChain::Nft => &self.nft_da_service, 
            AppChain::Payments => &self.payments_da_service
        };

        let block = match da_service
                .get_block_with_hash(pointer.block_hash)
                .await
            {
                Ok(i) => i,
                Err(e) => {
                    return Err(
                        anyhow!("Error fetching data {:?}",e)
                    ); // Retry the same height on error
                }
            };
        let height = pointer.tx_height - 1;

        println!("Da Height: {}", height);

        Ok(block.transactions[height].clone())
    }

    pub async fn submit_batch(&self, param: SubmitProofParam) -> Result<(), Error> {
        let tx = self.get_da_tx(param.da_tx_pointer.clone()).await?;
        let blob = tx.blob();

        //TODO: Check if all transactions are available and complete.

        match param.chain {
            AppChain::Nft => self.verify_nft_batch(param, blob),
            AppChain::Payments => self.verify_payments_batch(param, blob),
        }
    }

    pub fn verify_nft_batch(
        &self,
        param: SubmitProofParam,
        blob: &[u8],
    ) -> Result<(), Error> {
        let mut app_state = self.app_state.lock().unwrap();
        let da_batch: DABatch<NftTransaction> = match bincode::deserialize(blob) {
            Ok(i) => i, 
            Err(e) => return Err(anyhow!("Da batch deserialization failed due to error: {:?}", e))
        };
        let session_receipt: Receipt = match bincode::deserialize(&param.session_receipt) {
            Ok(i) => i,  
            Err(e) => return Err(anyhow!("proof deserialization failed due to error: {:?}", e))
        };

        // if da_batch.header != batch.header {
        //     return Err(anyhow!("Provided batch header does not match header posted to DA."));
        // }

        //TODO: Da validity check.

        println!("verifying NFT batch.");
        match session_receipt.verify(NFT_ID) {
            Ok(i) => Ok::<(), Error>(()),
            //TODO: Simplify this chaining.
            Err(e) => {
                return Err(anyhow!("Unable to verify proof."))
            }
        };

        println!("Verified NFT batch. Will be aggregated in the next cycle.");
        //Doing it this way to compare public parameters to submitted batch.
        let batch_header: BatchHeader = from_slice(&session_receipt.journal).unwrap();
        let last_batch_header: BatchHeader = app_state.get_last_nft_verified_batch();
        //TODO: change this to calculate root of all receipts, currently we assume
        //there is only one receipt; per batch.
        let receipts_root: H256 = param.receipts[0].to_h256();

        //TODO: Seperate the check for better error response.
        if receipts_root == batch_header.receipts_root
            && last_batch_header.state_root == batch_header.pre_state_root
        {
            app_state.verified_nft_batches.push(BatchWithReceipts {
                header: batch_header,
                receipts: param.receipts,
            });

            println!(
                "Added nft batch, total count: {:?}",
                app_state.verified_nft_batches.len()
            );

            Ok(())
        } else {
            println!(
                "Invalid proof receipts root: {:?} {:?}",
                &receipts_root, &batch_header.receipts_root
            );
            println!(
                "pre_state root: {:?} {:?}",
                &last_batch_header.state_root, &batch_header.pre_state_root
            );
            Err(anyhow!("Invalid proof."))
        }
    }

    pub fn verify_payments_batch(
        &self,
        param: SubmitProofParam,
        blob: &[u8],
    ) -> Result<(), Error> {
        let mut app_state = self.app_state.lock().unwrap();
        let session_receipt: Receipt = match bincode::deserialize(&param.session_receipt) {
            Ok(i) => i, 
            Err(e) => return Err(anyhow!("proof deserialization failed due to error: {:?}", e))
        };
        let da_batch: DABatch<PaymentsTransaction> = match bincode::deserialize(blob) {
            Ok(i) => i, 
            Err(e) => return Err(anyhow!("Da batch deserialization failed due to error: {:?}", e))
        };
        
        //TODO: Da validity check.

        println!("verifying payments batch.");
        match session_receipt.verify(PAYMENTS_ID) {
            Ok(i) => Ok::<(), Error>(()),
            //TODO: Simplify this chaining.
            Err(e) => {
                return Err(anyhow!("Unable to verify proof."))
            }
        };

        let batch_header: BatchHeader = from_slice(&session_receipt.journal).unwrap();
        let last_batch_header: BatchHeader = app_state.get_last_payments_verified_batch();
        //TODO: change this to calculate root of all receipts, currently we assume
        //there is only one receipt per batch.
        let receipts_root: H256 = param.receipts[0].to_h256();

        //TODO: Seperate the check for better error response.
        if receipts_root == batch_header.receipts_root
            && last_batch_header.state_root == batch_header.pre_state_root
        {
            app_state
                .verified_payments_batches
                .push(BatchWithReceipts {
                    header: batch_header,
                    receipts: param.receipts,
                });

            println!("Verified and added payments batch. Will be aggregated in the next cycle.");

            Ok(())
        } else {
            Err(anyhow!("Invalid proof."))
        }
    }
}

async fn submit_batch(
    service: web::Data<NexusApp>,
    call: web::Json<SubmitProofParam>,
) -> impl Responder {
    let deserialized_call: SubmitProofParam = call.into_inner();
    
    match service.submit_batch(deserialized_call).await {
        Ok(()) => "Proof verified and submitted successfully.", 
        Err(e) => "Proof submission failed."
    }
}

fn hex_string_to_u8_array(hex_string: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_string).unwrap();

    if bytes.len() != 32 {
        panic!("Hexadecimal string must represent exactly 32 bytes");
    }

    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);

    array
}

async fn get_receipt_with_proof(
    service: web::Data<NexusApp>,
    call: web::Query<ReceiptQuery>,
) -> impl Responder {
    let deserialized_call: ReceiptQuery = call.into_inner();
    let key: H256 = H256::from(hex_string_to_u8_array(&deserialized_call.key));
    let tree_state = service.tree_state.lock().unwrap();

    let receipt_with_proof = match tree_state.get_with_proof(&key) {
        Ok(i) => i,
        Err(e) => return HttpResponse::InternalServerError().body("Internal error."),
    };

    HttpResponse::Ok().json(receipt_with_proof)
}

async fn get_current_batch(service: web::Data<NexusApp>) -> impl Responder {
    let app_state = service.app_state.lock().unwrap();

    //TODO: Create method on App or app state to get this.
    let current_batch = app_state.last_aggregated_batch.clone();

    HttpResponse::Ok().json(current_batch)
}

pub async fn start_rpc_server(shared_service: NexusApp) -> impl Send {
    let json_cfg = web::JsonConfig::default()
    // limit request payload size
    .limit(1800000000);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_service.clone()))
            .app_data(json_cfg.clone())
            .route("/submit-batch", web::post().to(submit_batch))
            .route("/current-batch", web::get().to(get_current_batch))
            .route("/receipt", web::get().to(get_receipt_with_proof))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await;
}
