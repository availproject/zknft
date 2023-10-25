use crate::{
    traits::Leaf, 
    utils::hex_string_to_u8_array
};
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
#[cfg(any(feature = "native", feature = "native-metal"))]
use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::{
    traits::{Hasher, Value},
    MerkleProof, H256,
};

use parity_scale_codec::{Encode, Decode};
use ed25519_consensus::VerificationKey;
use ed25519_consensus::Signature;
use serde_big_array::BigArray;
#[cfg(any(feature = "native", feature = "native-metal"))]
use std::marker::PhantomData;
#[cfg(any(feature = "native", feature = "native-metal"))]
use http::status::StatusCode;
use std::convert::TryFrom;
use anyhow::{anyhow, Error};

#[derive(Default)]
pub struct ShaHasher(pub Sha256);

impl ShaHasher {
    pub fn new() -> Self {
        ShaHasher(Sha256::new())
    }
}
impl Hasher for ShaHasher {
    fn write_h256(&mut self, h: &H256) {
        self.0.update(h.as_slice())
    }

    fn write_byte(&mut self, b: u8) {
        self.0.update([b])
    }

    fn finish(self) -> H256 {
        let bytes = self.0.finalize();
        //TODO: Check if unwrap could be removed.
        let sha2_array: [u8; 32] = bytes.as_slice().try_into().unwrap();
        H256::from(sha2_array)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Commit {
    pub pre_state_root: String,
    pub post_state_root: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StateUpdate<S> {
    pub pre_state_root: H256,
    pub post_state_root: H256,
    pub pre_state_with_proof: (Vec<(H256, S)>, MerkleProof),
    pub post_state_with_proof: (Vec<(H256, S)>, MerkleProof),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BatchHeader {
    pub pre_state_root: H256,
    pub state_root: H256,
    pub transactions_root: H256,
    //Note: Receipts root is not required for security guarantees, but helps
    //nexus verify receipts list and update its tree.
    pub receipts_root: H256,
    pub batch_number: u64,
}

impl BatchHeader {
    pub fn default() -> Self {
        Self {
            pre_state_root: H256::zero(),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            batch_number: 0,
        }
    }
}

#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Debug, Deserialize, Serialize)]
pub struct DABatch <T> {
    pub header: BatchHeader,
    pub transactions: Vec<T>,
}

#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchWithProof<T> {
    pub header: BatchHeader,
    pub transaction_with_receipts: Vec<TransactionWithReceipt<T>>,
    pub proof: Receipt,
}

#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionWithReceipt<T> {
    pub transaction: T,
    pub receipt: TransactionReceipt,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Default, Encode, Decode)]
pub struct TransactionReceipt {
    pub chain_id: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TxSignature (
    #[serde(with = "BigArray")]
    [u8; 64]
);

impl TxSignature {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn as_signature(&self) -> Signature {
        Signature::from(self.0)
    }
}

impl From<[u8; 64]> for TxSignature {
    fn from(value: [u8; 64]) -> Self {
        Self(value)
    }
}

impl From<Signature> for TxSignature {
    fn from(s: Signature) -> Self {
        Self (s.to_bytes())
    }
}

impl TryFrom<&String> for TxSignature {
    type Error = anyhow::Error;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        // Parse the hexadecimal string into a [u8; 64]
        let bytes = hex::decode(s)?;

        if bytes.len() != 64 {
            return Err(anyhow!("Hexadecimal string must represent exactly 32 bytes"));
        }
    
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes);
    

        // Attempt to convert the bytes into a TxSignature
        Ok(TxSignature(array))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct Address(pub [u8; 32]);

impl Address {
    pub fn verification_key(&self) -> Result<VerificationKey, Error> {
        let mut key: [u8; 32] = Default::default();
        let public_key = self.0.as_slice();

        key.copy_from_slice(public_key);

        //TODO: Better error handling.
        Ok(VerificationKey::try_from(key)?)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [0; 32]
    }

    pub fn verify_msg(&self, sig: &TxSignature, msg: &[u8]) -> bool {
        let verification_key = match self.verification_key() {
            Ok(i) => i, 
            Err(e) => return false,
        };

        //TODO: Return error instead.
        match verification_key.verify(&sig.as_signature(), msg) {
            Ok(()) => true, 
            Err(_) => false, 
        }
    }

    pub fn get_key(&self) -> H256 {
        H256::from(self.0)
    }

    pub fn zero() -> Self {
        Self([0; 32])
    }
}

#[cfg(any(feature = "native", feature = "native-metal"))]
impl TryFrom<&String> for Address {
    type Error = anyhow::Error;

    fn try_from(hex_string: &String) -> Result<Self, Self::Error> {
        let address_array: [u8; 32] = match hex_string_to_u8_array(&hex_string) {
            Ok(i) => i, 
            Err(e) => return Err(e),
        };

        Ok(Self(address_array))
    }
}

impl TransactionReceipt {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl Value for TransactionReceipt {
    fn to_h256(&self) -> H256 {
        if self.chain_id == 0 && self.data == vec![0] {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let serialized = self.to_encoded();
        hasher.0.update(&serialized);

        hasher.finish()
    }

    fn zero() -> Self {
        Default::default()
    }
}

impl Leaf<H256> for TransactionReceipt {
    fn get_key(&self) -> H256 {
        self.to_h256()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct AggregatedBatch {
    pub proof_number: u64,
    pub receipts_root: H256,
}

#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DaTxPointer {
  pub block_hash: [u8; 32],
  pub hash: [u8; 32], 
  pub chain: AppChain,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AppChain {
    Nft,
    Payments,
}

#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitProofParam {
    pub session_receipt: Vec<u8>,
    pub receipts: Vec<TransactionReceipt>,
    pub chain: AppChain,
    pub da_tx_pointer: DaTxPointer,
}

#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Debug)]
pub enum ClientReply<T: Serialize> {
    Ok(T), 
    Error(anyhow::Error), 
    BadRequest,
}
#[cfg(any(feature = "native", feature = "native-metal"))]
impl <T: Send + Serialize> warp::Reply for ClientReply<T> {
    fn into_response(self) -> warp::reply::Response {
        match self {
            ClientReply::Ok(i) => warp::reply::with_status(
                    warp::reply::json(&i), 
                    StatusCode::OK,
            ).into_response(), 
            ClientReply::Error(e) => warp::reply::with_status(
				warp::reply::json(&e.to_string()),
				StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response(),
            ClientReply::BadRequest => warp::reply::with_status(
				warp::reply::json(&"Bad Request".to_owned()),
				StatusCode::BAD_REQUEST,
            ).into_response(),
        }
    }
}
