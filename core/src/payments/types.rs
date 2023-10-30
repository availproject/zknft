use crate::{
    traits::{Leaf, TxHasher},
    types::{ShaHasher, TxSignature, Address},
};
use risc0_zkvm::sha::rust_crypto::Digest;
use parity_scale_codec::{Encode, Decode};
use serde::{Deserialize, Serialize};
use ed25519_consensus::Signature;
use anyhow::{anyhow};
use sparse_merkle_tree::{
    traits::{Hasher, Value},
    H256,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct Account {
    pub address: Address,
    pub balance: u64,
    pub nonce: u64,
}

impl Leaf<H256> for Account {
    fn get_key(&self) -> H256 {
        self.address.get_key()
    }
}

impl Value for Account {
    fn to_h256(&self) -> H256 {
        if self.balance == 0 && self.nonce == 0 {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let encoded = self.encode();

        hasher.0.update(&encoded);
        hasher.finish()
    }

    fn zero() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub enum CallType {
    Transfer,
    Mint,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct Transaction {
    pub message: Vec<u8>,
    pub signature: TxSignature,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct TransactionMessage {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub call_type: CallType,
    pub data: Option<String>,
}

impl TxHasher for Transaction {
    fn to_h256(&self) -> H256 {
        let mut hasher = ShaHasher::new();
        let serialized = self.to_encoded();
        println!("Serialized tx: {:?}", &serialized);
        
        hasher.0.update(&serialized);

        hasher.finish()
    }
}

impl Transaction {
    pub fn signature(&self) -> Signature {
        Signature::from(*self.signature.as_bytes())
    }

    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl TransactionMessage {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct PaymentReceiptData {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub call_type: CallType,
    pub data: Option<String>,
    pub nonce: u64,
}

impl PaymentReceiptData {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl TryFrom<Transaction> for TransactionMessage {
    type Error = anyhow::Error;

    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        let vec_u8 = value.message;
        let mut slice_u8: &[u8] = &vec_u8;

        match TransactionMessage::decode(&mut slice_u8) {
            Ok(i) => Ok(i),
            Err(e) => Err(anyhow!("{:?}", e))
        }
    }
}
