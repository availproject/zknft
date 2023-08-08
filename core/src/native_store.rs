use bincode::{deserialize, serialize};
use rocksdb::{Error as RocksdbError, Options, DB};
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::error::Error;
use sparse_merkle_tree::traits::{StoreReadOps, StoreWriteOps};
use sparse_merkle_tree::BranchKey;
use sparse_merkle_tree::BranchNode;
use sparse_merkle_tree::H256;

//Store to be used inside StateMachine.
pub struct NativeStore {
    db: DB,
}

impl NativeStore {
    pub fn from_path(path: String) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).unwrap();

        NativeStore { db }
    }

    pub fn with_db(db: DB) -> Self {
        NativeStore { db }
    }

    pub fn db_asref(&self) -> &DB {
        &self.db
    }
}

impl<V: for<'a> Deserialize<'a>> StoreReadOps<V> for NativeStore {
    fn get_branch(&self, branch_key: &BranchKey) -> Result<Option<BranchNode>, Error> {
        let serialized_key = match serialize(&branch_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        match self.db.get(&serialized_key) {
            Err(e) => Err(Error::Store(e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some(i)) => Ok(deserialize(&i).unwrap()),
        }
    }
    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<V>, Error> {
        let key = leaf_key.as_slice();

        match self.db.get(&key) {
            Err(e) => Err(Error::Store(e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some(i)) => Ok(deserialize(&i).unwrap()),
        }
    }
}

impl<V: Serialize> StoreWriteOps<V> for NativeStore {
    fn insert_branch(&mut self, node_key: BranchKey, branch: BranchNode) -> Result<(), Error> {
        let serialized_key = match serialize(&node_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        match self
            .db
            .put(serialized_key, bincode::serialize(&branch).unwrap())
        {
            Err(e) => Err(Error::Store(e.to_string())),
            _ => Ok(()),
        }
    }
    fn insert_leaf(&mut self, leaf_key: H256, leaf: V) -> Result<(), Error> {
        match self
            .db
            .put(leaf_key.as_slice(), bincode::serialize(&leaf).unwrap())
        {
            Err(e) => Err(Error::Store(e.to_string())),
            _ => Ok(()),
        }
    }
    fn remove_branch(&mut self, node_key: &BranchKey) -> Result<(), Error> {
        let serialized_key = match serialize(&node_key) {
            Err(e) => return Err(Error::Store(e.to_string())),
            Ok(i) => i,
        };

        match self.db.get(&serialized_key) {
            Err(e) => Err(Error::Store(e.to_string())),
            Ok(Some(i)) => match self.db.delete(&serialized_key) {
                Err(e) => Err(Error::Store(e.to_string())),
                _ => Ok(()),
            },
            Ok(None) => Ok(()),
        }
    }
    fn remove_leaf(&mut self, leaf_key: &H256) -> Result<(), Error> {
      let serialized_key = leaf_key.as_slice();

      match self.db.get(&serialized_key) {
          Err(e) => Err(Error::Store(e.to_string())),
          Ok(Some(i)) => match self.db.delete(&serialized_key) {
              Err(e) => Err(Error::Store(e.to_string())),
              _ => Ok(()),
          },
          Ok(None) => Ok(()),
      }
    }
}
