use crate::{state::MerkleStore, traits::Leaf, types::StateUpdate};
use anyhow::{anyhow, Error};
use risc0_zkvm::sha::rust_crypto::{Digest as _, Sha256};
use rocksdb::{Options, DB};
use serde::{de::DeserializeOwned, Serialize};
use sparse_merkle_tree::{traits::Hasher, traits::Value, MerkleProof, SparseMerkleTree, H256};
use std::{
    cmp::PartialEq,
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    merkle_store: MerkleStore,
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
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db =
            DB::open(&db_options, String::from("./app_node")).expect("unable to open rocks db.");
        let cache: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let cache_arc = Arc::new(Mutex::new(cache));
        let db_arc = Arc::new(Mutex::new(db));

        VmState {
            tree: SparseMerkleTree::new(
                root,
                MerkleStore::with_db(db_arc.clone(), cache_arc.clone()),
            ),
            merkle_store: MerkleStore::with_db(db_arc, cache_arc),
        }
    }

    //Revert to last committed state and clear cache.
    pub fn revert(&mut self) -> Result<(), Error> {
        self.merkle_store.clear_cache()?;

        let tree = match SparseMerkleTree::new_with_store(self.merkle_store.clone()) {
            Ok(i) => i,
            Err(e) => {
                return Err(anyhow!(
                    "Could not calculate root from last committed state. Critical error. {e}"
                ))
            }
        };

        self.tree = tree;
        println!("Reverted to root: {:?}", self.tree.root());

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        match self.merkle_store.commit() {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!(e.to_string())),
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

    pub fn get(&self, key: &H256, committed: bool) -> Result<Option<V>, Error> {
        self.merkle_store
            .get(key.as_slice(), committed)
            .map_err(|e| anyhow!({ e }))
    }

    //Gets from state even if not committed.
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
