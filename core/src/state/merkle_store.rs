use rocksdb::{Options, DB};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use sparse_merkle_tree::error::Error;
use sparse_merkle_tree::traits::{StoreReadOps, StoreWriteOps};
use sparse_merkle_tree::BranchKey;
use sparse_merkle_tree::BranchNode;
use sparse_merkle_tree::H256;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

//Store to be used inside StateMachine to store Merkle Tree.
#[derive(Clone)]
pub struct MerkleStore {
    db: Arc<Mutex<DB>>,
    cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MerkleStore {
    pub fn with_db(db: Arc<Mutex<DB>>, cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>) -> Self {
        MerkleStore { db, cache }
    }

    pub fn get<V: DeserializeOwned>(
        &self,
        serialized_key: &[u8],
        committed: bool,
    ) -> Result<Option<V>, Error> {
        if !committed {
            let cache = match self.cache.lock() {
                Ok(i) => i,
                Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
            };

            match cache.get(serialized_key) {
                Some(i) => {
                    //Empty vectors mean the value was deleted.
                    if !i.is_empty() {
                        Ok(from_slice::<Option<V>>(&i).unwrap())
                    } else {
                        Ok(None)
                    }
                }
                None => {
                    let db = match self.db.lock() {
                        Ok(i) => i,
                        Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
                    };

                    match db.get(serialized_key) {
                        Ok(Some(i)) => {
                            println!("now trying to deserialize to value..");
                            let deserialized_value: V = match from_slice(&i) {
                                Ok(v) => v,
                                Err(e) => return Err(Error::Store(e.to_string())),
                            };

                            Ok(Some(deserialized_value))
                        }
                        Ok(None) => Ok(None),
                        Err(e) => {
                            println!("OHh nooo some error 🧐 {:?}", serialized_key);
                            Err(Error::Store(e.to_string()))
                        }
                    }
                }
            }
        } else {
            let db = match self.db.lock() {
                Ok(i) => i,
                Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
            };

            match db.get(serialized_key) {
                Ok(Some(i)) => {
                    println!("now trying to deserialize to value..");
                    let deserialized_value: V = match from_slice(&i) {
                        Ok(v) => v,
                        Err(e) => return Err(Error::Store(e.to_string())),
                    };

                    Ok(Some(deserialized_value))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(Error::Store(e.to_string())),
            }
        }
    }

    pub fn put<V: Serialize>(&self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
        };

        cache.insert(serialized_key.to_vec(), to_vec(value).unwrap());

        Ok(())
    }

    pub fn delete(&self, serialized_key: &[u8]) -> Result<(), Error> {
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
        };

        cache.remove(&serialized_key.to_vec());

        cache.insert(serialized_key.to_vec(), vec![]);

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        let db = match self.db.lock() {
            Ok(i) => i,
            Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
        };
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
        };
        println!("committing {}", cache.len());
        for (key, value) in cache.iter() {
            if !value.is_empty() {
                println!("Added to db: {:?}", &key);
                match db.put(key, value) {
                    Err(e) => return Err(Error::Store(e.to_string())),
                    _ => (),
                }
            } else {
                //Getting from underlying db below so as to not deserialise the
                //value as it is not required.
                match db.get(key) {
                    Err(e) => return Err(Error::Store(e.to_string())),
                    Ok(Some(_)) => match db.delete(key) {
                        Err(e) => return Err(Error::Store(e.to_string())),
                        _ => (),
                    },
                    Ok(None) => (),
                };
            }
        }

        cache.clear();
        Ok(())
    }

    pub fn clear_cache(&mut self) -> Result<(), Error> {
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(Error::Store(String::from("No lock obtained."))),
        };

        Ok(cache.clear())
    }
}

impl<V: DeserializeOwned> StoreReadOps<V> for MerkleStore {
    fn get_branch(&self, branch_key: &BranchKey) -> Result<Option<BranchNode>, Error> {
        let serialized_key = match to_vec(branch_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        self.get(&serialized_key, false)
    }

    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<V>, Error> {
        let key = leaf_key.as_slice();

        self.get(key, false)
    }
}

impl<V: Serialize> StoreWriteOps<V> for MerkleStore {
    fn insert_branch(&mut self, node_key: BranchKey, branch: BranchNode) -> Result<(), Error> {
        let serialized_key = match to_vec(&node_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        self.put(&serialized_key, &branch)
    }

    fn insert_leaf(&mut self, leaf_key: H256, leaf: V) -> Result<(), Error> {
        self.put(leaf_key.as_slice(), &leaf)
    }

    fn remove_branch(&mut self, node_key: &BranchKey) -> Result<(), Error> {
        let serialized_key = match to_vec(&node_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        self.delete(&serialized_key)
    }

    fn remove_leaf(&mut self, leaf_key: &H256) -> Result<(), Error> {
        let serialized_key = leaf_key.as_slice();

        self.delete(serialized_key)
    }
}
