use rocksdb::{Options, DB};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use sparse_merkle_tree::H256;
use anyhow::{Error, anyhow};

//Wrapper class to RocksDB which is used as backing storage.
pub struct NodeDB {
    db: DB,
}

impl NodeDB {
    pub fn from_path(path: String) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).unwrap();

        NodeDB { db }
    }

    pub fn with_db(db: DB) -> Self {
        NodeDB { db }
    }

    pub fn db_asref(&self) -> &DB {
        &self.db
    }

    pub fn get<V: DeserializeOwned>(&self, serialized_key: &[u8]) -> Result<Option<V>, Error> {
        match self.db.get(serialized_key) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some(i)) => Ok(from_slice(&i).unwrap()),
        }
    }

    pub fn put<V: Serialize>(&self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        match self.db.put(serialized_key, to_vec(&value).unwrap()) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            _ => Ok(()),
        }
    }

    pub fn delete(&self, serialized_key: &[u8]) -> Result<(), Error> {
        match self.db.get(serialized_key) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            Ok(Some(_)) => match self.db.delete(serialized_key) {
                Err(e) => Err(anyhow!("{}", e.to_string())),
                _ => Ok(()),
            },
            Ok(None) => Ok(()),
        }
    }

    pub fn get_current_root(&self) -> Result<Option<H256>, Error> {
        self.get(b"current-root")
    }

    pub fn set_current_root(&self, root: &H256) -> Result<(), Error> {
        self.put(b"current-root", root)
    }
}
