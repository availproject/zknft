use rocksdb::{Options, DB};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use sparse_merkle_tree::error::Error;
use sparse_merkle_tree::traits::{StoreReadOps, StoreWriteOps};
use sparse_merkle_tree::BranchKey;
use sparse_merkle_tree::BranchNode;
use sparse_merkle_tree::H256;

//Store to be used inside StateMachine to store Merkle Tree.
pub struct MerkleStore {
    db: DB,
}

impl MerkleStore {
    pub fn from_path(path: String) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).unwrap();

        MerkleStore { db }
    }

    pub fn with_db(db: DB) -> Self {
        MerkleStore { db }
    }

    pub fn db_asref(&self) -> &DB {
        &self.db
    }

    pub fn get<V: DeserializeOwned>(&self, serialized_key: &[u8]) -> Result<Option<V>, Error> {
        let value = match self.db.get(serialized_key) {
            Err(e) => Err(Error::Store(e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some(i)) => Ok(from_slice::<Option<V>>(&i).unwrap()),
        };

        value
    }

    pub fn put<V: Serialize>(&self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        match self.db.put(serialized_key, to_vec(&value).unwrap()) {
            Err(e) => Err(Error::Store(e.to_string())),
            _ => Ok(()),
        }
    }

    pub fn delete(&self, serialized_key: &[u8]) -> Result<(), Error> {
        match self.db.get(&serialized_key) {
            Err(e) => Err(Error::Store(e.to_string())),
            Ok(Some(_)) => match self.db.delete(&serialized_key) {
                Err(e) => Err(Error::Store(e.to_string())),
                _ => Ok(()),
            },
            Ok(None) => Ok(()),
        }
    }
}

impl<V: DeserializeOwned> StoreReadOps<V> for MerkleStore {
    fn get_branch(&self, branch_key: &BranchKey) -> Result<Option<BranchNode>, Error> {
        let serialized_key = match to_vec(branch_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        self.get(&serialized_key)
    }

    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<V>, Error> {
        let key = leaf_key.as_slice();

        self.get(&key)
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

        self.delete(&serialized_key)
    }
}
