use rocksdb::{Error as RocksdbError, Options, DB};

use crate::zk_storage::NftTable;
use fuel_merkle::storage::{Mappable, StorageInspect, StorageMutate};
use std::borrow::Cow;

pub struct NativeStorage {
    db: DB,
}

impl NativeStorage {
    pub fn from_path(path: String) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).unwrap();

        NativeStorage { db }
    }

    pub fn with_db(db: DB) -> Self {
        NativeStorage { db }
    }

    pub fn db_asref(&self) -> &DB {
        &self.db
    }
}

impl StorageInspect<NftTable> for NativeStorage {
    type Error = RocksdbError;
    fn get(
        &self,
        key: &<NftTable as Mappable>::Key,
    ) -> Result<Option<Cow<'_, <NftTable as Mappable>::OwnedValue>>, Self::Error> {
        match self.db.get(key) {
            Err(e) => Err(e),
            Ok(Some(i)) => Ok(Some(Cow::Owned(bincode::deserialize(&i).unwrap()))),
            Ok(None) => Ok(None),
        }
    }

    fn contains_key(&self, key: &<NftTable as Mappable>::Key) -> Result<bool, Self::Error>
    where
        <NftTable as Mappable>::Key: AsRef<[u8]>,
    {
        match self.db.get(key) {
            Err(e) => Err(e),
            Ok(Some(_i)) => Ok(true),
            Ok(None) => Ok(false),
        }
    }
}

impl StorageMutate<NftTable> for NativeStorage {
    fn insert(
        &mut self,
        key: &<NftTable as Mappable>::Key,
        value: &<NftTable as Mappable>::Value,
    ) -> Result<
        Option<<NftTable as Mappable>::OwnedValue>,
        <NativeStorage as StorageInspect<NftTable>>::Error,
    > {
        match self.db.put(key, bincode::serialize(&value).unwrap()) {
            Err(e) => Err(e),
            _ => Ok(Some(*value)),
        }
    }

    fn remove(
        &mut self,
        key: &<NftTable as Mappable>::Key,
    ) -> Result<
        Option<<NftTable as Mappable>::OwnedValue>,
        <NativeStorage as StorageInspect<NftTable>>::Error,
    > {
        match self.db.get(key) {
            Err(e) => Err(e),
            Ok(Some(i)) => match self.db.delete(key) {
                Err(e) => Err(e),
                _ => Ok(Some(bincode::deserialize(&i).unwrap())),
            },
            Ok(None) => Ok(None),
        }
    }
}
