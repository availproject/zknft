// use sparse_merkle_tree::{
//   blake2b::Blake2bHasher, default_store::DefaultStore,
//   error::Error, MerkleProof,
//   SparseMerkleTree, traits::Value, H256
// };
use crate::{
    zk_storage::{NftTable},
    Nft, NftId,
};
use fuel_merkle::common::Bytes32;
use fuel_merkle::sparse::MerkleTree;
use fuel_merkle::sparse::MerkleTreeError;
use fuel_merkle::sparse::MerkleTreeKey;
use fuel_merkle::sparse::Primitive;
use fuel_merkle::storage::{Mappable, StorageInspect, StorageMutate};
use primitive_types::U256;
// use rocksdb::Error as RocksdbError;
// use rocksdb::DB;
use std::fmt;
//Placeholder error to remove
use core::fmt::Error as FMTError;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct MyError {
    msg: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

pub struct StateMachine<StorageType: StorageInspect<NftTable> + StorageMutate<NftTable>> {
    pub state: MerkleTree<NftTable, StorageType>,
  //  store: DB,
}

impl<Error: std::fmt::Debug, StorageType: StorageInspect<NftTable, Error = Error> + StorageMutate<NftTable>>
    StateMachine<StorageType>
{
    pub fn new(store: StorageType) -> Self {
        // let general_store = DB::open_default("./demo_data/2").expect("Unable to find DB.");

        //Code to add NFT
        let nft1 = Nft {
            id: NftId(U256::from_dec_str("1").unwrap()),
            owner: String::from("ABCD"),
        };
        let nft2 = Nft {
            id: NftId(U256::from_dec_str("2").unwrap()),
            owner: String::from("EFGH"),
        };

        let serialized_nft = bincode::serialize(&nft1).unwrap();
        let serialized_nft2 = bincode::serialize(&nft2).unwrap();

        let mut bytes = [0u8; 32];
        nft1.id.0.to_big_endian(&mut bytes[..]);

        let mut bytes2 = [0u8; 32];
        nft2.id.0.to_big_endian(&mut bytes2[..]);

        let set: Vec<(MerkleTreeKey, &[u8])> = unsafe {
            vec![
                (MerkleTreeKey::convert(bytes), &serialized_nft),
                (MerkleTreeKey::convert(bytes2), &serialized_nft2),
            ]
        };

        let state = MerkleTree::from_set(store, set.into_iter()).unwrap();

        StateMachine {
            state,
        //    store: general_store,
        }
    }

    pub fn load(store: StorageType) -> Self {
        // let general_store = DB::open_default("./demo_data/2").expect("Unable to find DB.");
        let bytes: [u8; 32] = [127, 116, 122, 219, 223, 166, 3, 8, 126, 27, 73, 169, 153, 127, 141, 212, 184, 249, 23, 184, 124, 166, 180, 187, 129, 174, 230, 85, 188, 240, 207, 115];
        let state = MerkleTree::<NftTable, StorageType>::load(store, &bytes).unwrap();

        StateMachine {
            state,
        //    store: general_store,
        }
    }

    pub fn transfer(
        &mut self,
        new_owner: String,
        nft_id: String,
        from: String,
    ) -> Result<(), MyError> {
        let id_256 = U256::from_dec_str(&nft_id).unwrap();
        let mut nft1 = Nft {
            id: NftId(U256::from_dec_str("1").unwrap()),
            owner: String::from("ABCD"),
        };
        let mut nft2 = Nft {
            id: NftId(U256::from_dec_str("2").unwrap()),
            owner: String::from("EFGH"),
        };

        let mut vec: Vec<&mut Nft> = vec![&mut nft1, &mut nft2];

        let mut nft_to_transfer: &mut Nft = match vec.iter_mut().find(|nft| nft.id.0 == id_256) {
            Some(i) => i,
            None => panic!("NFT not found."),
        };

        println!("Owner: {:?} From: {:?}", nft_to_transfer.owner, from);

        if nft_to_transfer.owner != from {
            panic!("Not owner");
        }

        nft_to_transfer.owner = new_owner;
        let mut bytes = [0u8; 32];
        id_256.to_big_endian(&mut bytes[..]);
        let serialized_nft = bincode::serialize(&nft_to_transfer).unwrap();
        
        match unsafe {
            self.state
                .update(MerkleTreeKey::convert(bytes), &serialized_nft)
        } {
            Ok(()) => Ok(()),
            Err(e) => panic!("Merkle update failed"),
        }
    }
}
