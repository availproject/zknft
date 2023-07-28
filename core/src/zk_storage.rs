#[cfg(feature = "native")]
use rocksdb::{IteratorMode, DB};
use core::fmt::Error;
use fuel_merkle::storage::{Mappable, StorageInspect, StorageMutate};
use std::borrow::Cow;
use fuel_merkle::common::Bytes32;
use fuel_merkle::sparse::Primitive;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct NftTable;

impl Mappable for NftTable {
    /// The `[u8; 32]` is a primitive type, so we can't optimize it more.
    type Key = Self::OwnedKey;
    type OwnedKey = Bytes32;

    type Value = Primitive;
    type OwnedValue = Primitive;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct ZkStorage {
    hashmap: HashMap<Vec<u8>, Vec<u8>>,
}

impl ZkStorage {
    #[cfg(feature = "native")]
    pub fn with_db(db: &DB) -> Self {
        let mut hashmap: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        // Retrieve an iterator over all key-value pairs in RocksDB
        let iterator = db.iterator(IteratorMode::Start);

        // Iterate over the key-value pairs and insert them into the HashMap
        for item in iterator {
            let (key, value) = item.unwrap();

            let key_vec = key.to_vec();
            let value_vec = value.to_vec();
            hashmap.insert(key_vec, value_vec);
        }

        ZkStorage { hashmap }
    }

    pub fn new () -> Self {
      let mut hashmap: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

      ZkStorage { hashmap }
    }
}

impl StorageInspect<NftTable> for ZkStorage {
    type Error = Error;
    fn get(
        &self,
        key: &<NftTable as Mappable>::Key,
    ) -> Result<Option<Cow<'_, <NftTable as Mappable>::OwnedValue>>, Self::Error> {
        match self.hashmap.get(&key.to_vec()) {
            Some(i) => Ok(Some(Cow::Owned(bincode::deserialize(i).unwrap()))),
            None => Ok(None),
        }
    }

    fn contains_key(&self, key: &<NftTable as Mappable>::Key) -> Result<bool, Self::Error>
    where
        <NftTable as Mappable>::Key: AsRef<[u8]>,
    {
        match self.hashmap.get(&key.to_vec()) {
            Some(_i) => Ok(true),
            None => Ok(false),
        }
    }
}

impl StorageMutate<NftTable> for ZkStorage {
    fn insert(
        &mut self,
        key: &<NftTable as Mappable>::Key,
        value: &<NftTable as Mappable>::Value,
    ) -> Result<
        Option<<NftTable as Mappable>::OwnedValue>,
        <ZkStorage as StorageInspect<NftTable>>::Error,
    > {
        match self
            .hashmap
            .insert(key.to_vec(), bincode::serialize(&value).unwrap())
        {
            Some(_i) => Ok(Some(*value)),
            None => Ok(None),
        }
    }

    fn remove(
        &mut self,
        key: &<NftTable as Mappable>::Key,
    ) -> Result<
        Option<<NftTable as Mappable>::OwnedValue>,
        <ZkStorage as StorageInspect<NftTable>>::Error,
    > {
        match self.hashmap.get(&key.to_vec()) {
            Some(_i) => match self.hashmap.remove(&key.to_vec()) {
                Some(r) => Ok(Some(bincode::deserialize(&r).unwrap())),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }
}
