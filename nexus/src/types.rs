use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use nft_core::types::TransactionReceipt;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DaTxPointer {
  pub block_hash: [u8; 32],
  pub tx_height: usize, 
  pub chain: AppChain,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AppChain {
    Nft,
    Payments,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitProofParam {
    pub session_receipt: Vec<u8>,
    pub receipts: Vec<TransactionReceipt>,
    pub chain: AppChain,
    pub da_tx_pointer: DaTxPointer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct ReceiptQuery {
    pub key: String,
}
