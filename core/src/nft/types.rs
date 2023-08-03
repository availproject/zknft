use crate::{traits::Leaf, types::ShaHasher};
use primitive_types::U256;
use risc0_zkvm::sha::rust_crypto::Digest;
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::{
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

impl Value for Nft {
    fn to_h256(&self) -> H256 {
        if self.owner.is_empty() {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let serialized = bincode::serialize(&self).unwrap();
        hasher.0.update(&serialized);
        let hash = hasher.finish();

        hash
    }

    fn zero() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Nft {
    pub id: NftId,
    pub owner: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CallType {
    Transfer,
    Mint,
    Burn,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NftCallParams {
    pub id: NftId,
    pub owner: Option<String>,
    pub from: String,
    pub call_type: CallType,
}
