use crate::{traits::Leaf, types::ShaHasher};
use risc0_zkvm::sha::rust_crypto::Digest;
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::{
    traits::{Hasher, Value},
    H256,
};

pub type Hash256 = [u8; 32];
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Address(pub Hash256);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Account {
    pub address: Address,
    pub balance: u64,
}

impl Leaf<H256> for Account {
    fn get_key(&self) -> H256 {
        self.address.get_key()
    }
}

impl Address {
    pub fn get_key(&self) -> H256 {
        H256::from(self.0)
    }
}

impl Value for Account {
    fn to_h256(&self) -> H256 {
        if self.balance == 0 {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CallType {
  Transfer, 
  Mint
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallParams {
  pub from: Address, 
  pub to: Address, 
  pub amount: u64, 
  pub call_type: CallType
}
