use crate::{
    traits::{Leaf, TxHasher},
    types::{ShaHasher, TransactionReceipt},
};
use primitive_types::U256;
use risc0_zkvm::sha::rust_crypto::Digest;
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::{
    merkle_proof::MerkleProof,
    traits::{Hasher, Value},
    H256,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct NftId(pub U256);

impl NftId {
    pub fn get_key(&self) -> H256 {
        let mut bytes = [0u8; 32];
        self.0.to_big_endian(&mut bytes[..]);

        H256::from(bytes)
    }
}

impl Leaf<H256> for Nft {
    fn get_key(&self) -> H256 {
        self.id.get_key()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Nft {
    pub id: NftId,
    pub owner: String,
    pub future: Option<Future>,
    pub nonce: u64,
}

impl Value for Nft {
    fn to_h256(&self) -> H256 {
        if self.owner.is_empty() {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Future {
    pub to: String,
    pub commitment: H256,
}

// impl Future {
//     fn to_h256(&self) -> H256 {
//         let mut hasher = ShaHasher::new();
//         let serialized = bincode::serialize(&self.0).unwrap();
//         hasher.0.update(&serialized);

//         hasher.finish()
//     }
// }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Transfer {
    pub id: NftId,
    pub to: String,
    //All from to be replaced by signatures
    pub from: String,
    pub data: Option<String>,
    pub future_commitment: Option<H256>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Mint {
    pub id: NftId,
    pub from: String,
    pub to: String,
    pub data: Option<String>,
    pub future_commitment: Option<H256>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Burn {
    pub id: NftId,
    pub from: String,
    pub data: Option<String>,
    pub future_commitment: Option<H256>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Trigger {
    pub id: NftId,
    pub from: String,
    pub data: Option<String>,
    pub merkle_proof: MerkleProof,
    pub receipt: TransactionReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NftTransaction {
    Transfer(Transfer),
    Mint(Mint),
    Burn(Burn),
    Trigger(Trigger),
}

impl TxHasher for NftTransaction {
    fn to_h256(&self) -> H256 {
        let mut hasher = ShaHasher::new();
        let serialized = bincode::serialize(&self).unwrap();
        hasher.0.update(&serialized);

        hasher.finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FutureReceiptData {
    pub id: NftId,
    pub from: String,
    pub to: String,
    pub future_commitment: H256,
    pub data: Option<String>,
    pub nonce: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransferReceiptData {
    pub id: NftId,
    pub from: String,
    pub to: String,
    pub data: Option<String>,
    pub nonce: u64,
}

impl TransferReceiptData {
    pub fn to_vec(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

impl FutureReceiptData {
    pub fn to_vec(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
