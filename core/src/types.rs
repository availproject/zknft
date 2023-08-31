use crate::traits::Leaf;
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
#[cfg(feature = "native")]
use risc0_zkvm::SessionReceipt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sparse_merkle_tree::{
    traits::{Hasher, Value},
    MerkleProof, H256,
};

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

#[cfg(feature = "native")]
#[derive(Debug, Deserialize, Serialize)]
pub struct Batch<T> {
    pub header: BatchHeader,
    pub transactions: Vec<T>,
}

#[cfg(feature = "native")]
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionWithReceipt<T> {
    pub transaction: T,
    pub receipt: TransactionReceipt,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Default)]
pub struct TransactionReceipt {
    pub chain_id: u64,
    pub data: Vec<u8>,
}

// impl TransactionReceipt {
//     pub fn to_h256(&self) -> H256 {
//         let mut hasher = ShaHasher::new();
//         let serialized = bincode::serialize(&self).unwrap();
//         hasher.0.update(&serialized);

//         hasher.finish()
//     }
// }

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
