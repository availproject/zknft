use crate::errors::{AppError, Error, ProofError};
use hex;
use nft_core::traits::Leaf;
use nft_core::{
    db::NodeDB,
    state::VmState,
    types::{BatchHeader, TransactionReceipt, BatchWithProof},
    nft::types::NftTransaction, 
    payments::types::Transaction as PaymentsTransaction,
};
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;
use std::thread;
use std::time::Duration;
use serde_json::{from_slice as from_json_slice, to_vec as to_json_vec};

//Below imports for HTTP server.
use actix_web::error;
use actix_web::rt::System;
use actix_web::HttpResponse;
use actix_web::{get, web, App, HttpServer, Responder};
use avail::avail::AvailBlock;
use avail::service::DaProvider;
use nft_methods::TRANSFER_ID as NFT_ID;
use payments_methods::TRANSFER_ID as PAYMENTS_ID;
use risc0_zkvm::{serde::from_slice, SessionReceipt};
use std::sync::{Arc, Mutex};

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
                None => unreachable!("Will not be empty in else."),
            }
        }
    }

    pub fn get_last_payments_verified_batch(&self) -> BatchHeader {
        if self.verified_payments_batches.proof_count() == 0 {
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
        let da_last_height = {
            let mut db = self.db.lock().unwrap();
            match db.get::<u64>(b"last_da_block") {
                Ok(Some(i)) => i,
                Ok(None) => self.da_start_height,
                Err(e) => panic!("Could not start node. {:?}", e),
            }
        };

        loop {
            let filtered_nft_da_block = match self
                .nft_da_service
                .get_finalized_at(self.da_start_height)
                .await
            {
                Ok(i) => i,
                Err(e) => {
                    println!(
                        "Error fetching data at height {}: {:?}",
                        self.da_start_height, e
                    );
                    continue; // Retry the same height on error
                }
            };

            let filtered_payments_da_block = match self
                .payments_da_service
                .get_finalized_at(self.da_start_height)
                .await
            {
                Ok(i) => i,
                Err(e) => {
                    println!(
                        "Error fetching data at height {}: {:?}",
                        self.da_start_height, e
                    );
                    continue; // Retry the same height on error
                }
            };

            if filtered_nft_da_block.transactions.len() > 0
                || filtered_payments_da_block.transactions.len() > 0
            {
                println!("Found tx blobs, aggregating batch.");
                // TODO: Check for errors
                self.verify_and_update_proofs(filtered_nft_da_block, filtered_payments_da_block);
            } else {
                println!("Empty txs in block number: {}", self.da_start_height);
            }

            // Move to the next height for the next iteration
            self.da_start_height += 1;

            let mut db = self.db.lock().unwrap();
            match db.put::<u64>(b"last_da_block", &self.da_start_height) {
                Ok(()) => (),
                Err(e) => println!("Warning: Could not save last processed block. error: {:?}", e),
            }
        }
    }

    fn verify_and_update_proofs(
        &mut self,
        filtered_nft_da_block: AvailBlock,
        filtered_payments_da_block: AvailBlock,
    ) -> () {
        let mut app_state = self.app_state.lock().unwrap();
        let mut tree_state = self.tree_state.lock().unwrap();
        let mut db = self.db.lock().unwrap();

        let pending_nft_batch_count = filtered_nft_da_block.transactions.len();
        let pending_payments_batch_count = filtered_payments_da_block.transactions.len();
        println!(
            "aggregating proofs: {:?}, {:?}",
            pending_nft_batch_count, pending_payments_batch_count
        );

        println!("Current root is: {:?}", &app_state.last_aggregated_batch);

        let mut receipts_to_add: Vec<TransactionReceipt> = vec![];

        for height in 0..pending_nft_batch_count {
            let next_batch = match from_json_slice::<BatchWithProof<NftTransaction>>(
                filtered_nft_da_block.transactions[height].blob(),
            ) {
                Ok(i) => i,
                //Ignoring tx blob if deserialisation failed.
                Err(e) => continue,
            };

            let mut tx_receipts: Vec<TransactionReceipt> = vec![];
            for tx in &next_batch.transaction_with_receipts {
                tx_receipts.push(tx.receipt.clone());
            }

            match self.verify_nft_batch(next_batch.proof, tx_receipts.clone()) {
                Ok(i) => i,
                //TODO: Log rejected batches.
                Err(e) => continue,
            }

            receipts_to_add.extend(tx_receipts);

            if height == pending_payments_batch_count - 1 {
                //TODO: Below should not be changed before final aggregation.
                app_state.last_aggregated_nft_batch = next_batch.header.clone();
            }
        }

        let pending_payments_batch_count = app_state.verified_payments_batches.proof_count();

        for height in 0..pending_payments_batch_count {
            let next_batch = match from_json_slice::<BatchWithProof<PaymentsTransaction>>(
                filtered_payments_da_block.transactions[height].blob(),
            ) {
                Ok(i) => i,
                //Ignoring tx blob if deserialisation failed.
                Err(e) => continue,
            };

            let mut tx_receipts: Vec<TransactionReceipt> = vec![];
            for tx in &next_batch.transaction_with_receipts {
                tx_receipts.push(tx.receipt.clone());
            }

            match self.verify_payments_batch(next_batch.proof, tx_receipts.clone()) {
                Ok(i) => i,
                //TODO: Log rejected batches.
                Err(e) => continue,
            }

            receipts_to_add.extend(tx_receipts);

            if height == pending_payments_batch_count - 1 {
                //TODO: Below should not be changed before final aggregation.
                app_state.last_aggregated_payments_batch = next_batch.header.clone();
            }
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
    }

    pub fn verify_nft_batch(
        &self,
        receipt: SessionReceipt,
        tx_receipts: Vec<TransactionReceipt>,
    ) -> Result<(), Error> {
        let mut app_state = self.app_state.lock().unwrap();
        println!("verifying NFT batch.");
        match receipt.verify(NFT_ID) {
            Ok(i) => Ok::<(), ProofError>(()),
            //TODO: Simplify this chaining.
            Err(e) => {
                return Err(Error::ProofError(ProofError(String::from(
                    "Unable to verify proof",
                ))))
            }
        };

        println!("Verified NFT batch. Will be aggregated in the next cycle.");
        let batch_header: BatchHeader = from_slice(&receipt.journal).unwrap();
        let last_batch_header: BatchHeader = app_state.get_last_nft_verified_batch();
        //TODO: change this to calculate root of all receipts, currently we assume
        //there is only one receipt per batch.
        let receipts_root: H256 = tx_receipts[0].to_h256();

        //TODO: Seperate the check for better error response.
        if receipts_root == batch_header.receipts_root
            && last_batch_header.state_root == batch_header.pre_state_root
        {
            app_state.verified_nft_batches.add_batch(BatchWithReceipts {
                header: batch_header,
                receipts: tx_receipts,
            });

            println!(
                "Added nft batch, total count: {:?}",
                app_state.verified_nft_batches.proof_count()
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
            Err(Error::ProofError(ProofError(String::from("Invalid proof"))))
        }
    }

    pub fn verify_payments_batch(
        &self,
        receipt: SessionReceipt,
        tx_receipts: Vec<TransactionReceipt>,
    ) -> Result<(), Error> {
        let mut app_state = self.app_state.lock().unwrap();
        println!("Verifying payments batch");

        match receipt.verify(PAYMENTS_ID) {
            Ok(i) => Ok::<(), ProofError>(()),
            //TODO: Simplify this chaining.
            Err(e) => {
                return Err(Error::ProofError(ProofError(String::from(
                    "Unable to verify proof",
                ))))
            }
        };

        let batch_header: BatchHeader = from_slice(&receipt.journal).unwrap();
        let last_batch_header: BatchHeader = app_state.get_last_payments_verified_batch();
        //TODO: change this to calculate root of all receipts, currently we assume
        //there is only one receipt per batch.
        let receipts_root: H256 = tx_receipts[0].to_h256();

        //TODO: Seperate the check for better error response.
        if receipts_root == batch_header.receipts_root
            && last_batch_header.state_root == batch_header.pre_state_root
        {
            app_state
                .verified_payments_batches
                .add_batch(BatchWithReceipts {
                    header: batch_header,
                    receipts: tx_receipts,
                });

            println!("Verified and added payments batch. Will be aggregated in the next cycle.");

            Ok(())
        } else {
            Err(Error::ProofError(ProofError(String::from("Invalid proof"))))
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
    chain: AppChain,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct ReceiptQuery {
    key: String,
}

async fn submit_batch(
    service: web::Data<NexusApp>,
    call: web::Json<SubmitProofParam>,
) -> impl Responder {
    let deserialized_call: SubmitProofParam = call.into_inner();
    //let mut app = service.lock().unwrap();

    println!(
        "Received request to verify and add batch for chain: {}",
        match deserialized_call.chain {
            AppChain::Nft => "NFT Chain",
            AppChain::Payments => "Payments Chain",
        }
    );

    match deserialized_call.chain {
        AppChain::Nft => match service.verify_nft_batch(
            deserialized_call.session_receipt,
            deserialized_call.receipts,
        ) {
            Ok(()) => "Proof Submitted",
            Err(e) => "Proof not submitted",
        },
        AppChain::Payments => match service.verify_payments_batch(
            deserialized_call.session_receipt,
            deserialized_call.receipts,
        ) {
            Ok(()) => "Proof Submitted",
            Err(e) => "Proof not submitted",
        },
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
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_service.clone()))
            .route("/submit-batch", web::post().to(submit_batch))
            .route("/current-batch", web::get().to(get_current_batch))
            .route("/receipt", web::get().to(get_receipt_with_proof))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await;
}
