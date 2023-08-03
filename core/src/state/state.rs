use crate::{
    errors::{Error, StateError},
    traits::Leaf,
    types::StateUpdate,
};
use risc0_zkvm::sha::rust_crypto::{Digest as _, Sha256};
use sparse_merkle_tree::{
    default_store::DefaultStore,
    traits::Value,
    traits::{Hasher}, SparseMerkleTree, H256,
};
use std::cmp::PartialEq;

#[derive(Default)]
pub struct ShaHasher(pub Sha256);
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

pub struct State<V: Value> {
    tree: SparseMerkleTree<ShaHasher, V, DefaultStore<V>>,
}

impl<V: Value + std::default::Default + Clone + Leaf<H256> + PartialEq> State<V> {
    pub fn new() -> Self {
        State {
            tree: SparseMerkleTree::default(),
        }
    }

    //TODO - Load a tree for persistent storage.

    pub fn update_set(&mut self, set: Vec<V>) -> Result<StateUpdate<V>, Error> {
        let pre_state_root = self.get_root();
        let pre_merkle_proof = self
            .tree
            .merkle_proof(set.iter().map(|v| v.get_key()).collect())
            .unwrap();
        let pre_merkle_set = set
            .iter()
            .map(|v| self.tree.get(&v.get_key()).expect("Cannot get from tree."))
            .collect();

        self.tree
            .update_all(set.clone().into_iter().map(|v| (v.get_key(), v)).collect())
            .unwrap();

        let post_state_root = self.get_root();
        let post_merkle_proof = self
            .tree
            .merkle_proof(set.iter().map(|v| v.get_key()).collect())
            .unwrap();

        Ok(StateUpdate {
            pre_state_root,
            post_state_root,
            pre_state_with_proof: (pre_merkle_set, pre_merkle_proof),
            post_state_with_proof: (set, post_merkle_proof),
        })
    }

    pub fn get(&self, key: &H256) -> Result<Option<V>, Error> {
        match self.tree.get(key) {
            Ok(i) => {
                if i == V::zero() {
                    Ok(None)
                } else {
                    Ok(Some(i))
                }
            }
            Err(_e) => Err(Error::from(StateError::ErroneousState)),
        }
    }

    pub fn get_root(&self) -> H256 {
        *self.tree.root()
    }
}
