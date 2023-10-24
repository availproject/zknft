use crate::{
    state::MerkleStore,
    traits::Leaf,
    types::StateUpdate,
};
use risc0_zkvm::sha::rust_crypto::{Digest as _, Sha256};
use serde::{de::DeserializeOwned, Serialize};
use sparse_merkle_tree::{
    traits::Hasher,
    traits::{Value},
    MerkleProof, SparseMerkleTree, H256,
};
use std::cmp::PartialEq;
use anyhow::{Error, anyhow};

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

//TODO - Replace MerkleStore with a generic so any backing store could be used.
pub struct VmState<V> {
    tree: SparseMerkleTree<ShaHasher, V, MerkleStore>,
}

impl<
        V: Value
            + std::default::Default
            + Clone
            + Leaf<H256>
            + PartialEq
            + DeserializeOwned
            + Serialize
            + std::fmt::Debug,
    > VmState<V>
{
    pub fn new(root: H256) -> Self {
        VmState {
            tree: SparseMerkleTree::new(root, MerkleStore::from_path(String::from("./app_node"))),
        }
    }

    pub fn update_set(&mut self, set: Vec<V>) -> Result<StateUpdate<V>, Error> {
        let pre_state_root = self.get_root();
        let pre_merkle_proof = self
            .tree
            .merkle_proof(set.iter().map(|v| v.get_key()).collect())
            .unwrap();

        let pre_merkle_set = set
            .iter()
            .map(|v| {
                (
                    v.get_key(),
                    self.tree.get(&v.get_key()).expect("Cannot get from tree."),
                )
            })
            .collect();

        self.tree
            .update_all(set.clone().into_iter().map(|v| (v.get_key(), v)).collect())
            .unwrap();

        let post_state_root = self.get_root();

        let post_merkle_set = set.iter().map(|v| (v.get_key(), v.clone())).collect();
        let post_merkle_proof = self
            .tree
            .merkle_proof(set.iter().map(|v| v.get_key()).collect())
            .unwrap();

        //println!("Pre: {:?} || Post {:?}", pre_merkle_proof, post_merkle_proof);

        Ok(StateUpdate {
            pre_state_root,
            post_state_root,
            pre_state_with_proof: (pre_merkle_set, pre_merkle_proof),
            post_state_with_proof: (post_merkle_set, post_merkle_proof),
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
            Err(_e) => Err(anyhow!("Erroneous state.")),
        }
    }

    pub fn get_with_proof(&self, key: &H256) -> Result<(V, MerkleProof), Error> {
        let value = match self.tree.get(key) {
            Ok(i) => i,
            Err(_e) => return Err(anyhow!("Erroneous state.")),
        };

        let proof = match self.tree.merkle_proof(vec![*key]) {
            Ok(i) => i,
            Err(_e) => return Err(anyhow!("Erroneous state.")),
        };

        Ok((value, proof))
    }

    pub fn get_root(&self) -> H256 {
        *self.tree.root()
    }
}
