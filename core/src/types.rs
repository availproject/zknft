use crate::traits::Leaf;
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
#[cfg(any(feature = "native", feature = "native-metal"))]
use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::{
    traits::{Hasher, Value},
    MerkleProof, H256,
};
use ed25519_consensus::Signature;
use ed25519_consensus::VerificationKey;
use serde_big_array::BigArray;

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

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Default)]
pub struct TransactionReceipt {
    pub chain_id: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxSignature (
    #[serde(with = "BigArray")]
    [u8; 64]
);

impl TxSignature {
    pub fn from(s: Signature) -> Self {
        Self (s.to_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn as_signature(&self) -> Signature {
        Signature::from(self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Address(pub H256);

impl Address {
    pub fn verification_key(&self) -> VerificationKey {
        let mut key: [u8; 32] = Default::default();
        let public_key = self.0.as_slice();

        key.copy_from_slice(public_key);

        //TODO: Better error handling.
        VerificationKey::try_from(key).unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.0 == H256::zero()
    }

    pub fn verify_msg(&self, sig: &TxSignature, msg: &[u8]) -> bool {
        let verification_key = self.verification_key();

        //TODO: Return error instead.
        match verification_key.verify(&sig.as_signature(), msg) {
            Ok(()) => true, 
            Err(_) => false, 
        }
    }

    pub fn get_key(&self) -> H256 {
        self.0
    }

    pub fn zero() -> Self {
        Self(H256::zero())
    }
}

impl Value for TransactionReceipt {
    fn to_h256(&self) -> H256 {
        if self.chain_id == 0 && self.data == vec![0] {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let serialized = bincode::serialize(&self).unwrap();
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
  pub tx_height: u32, 
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
