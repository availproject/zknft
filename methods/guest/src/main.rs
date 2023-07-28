#![no_main]
use nft_core::{ Nft, UserRequest, Commit, zk_storage::ZkStorage, state_machine::StateMachine };
use rs_merkle::{MerkleTree, algorithms::Sha256, Hasher};
use serde_json;

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let zk_storage: ZkStorage = env::read();
    // let zk_storage: ZkStorage = ZkStorage::new();
    let mut state_machine = StateMachine::load(zk_storage);

    state_machine.transfer(
        String::from("ABCD"),
        String::from("2"),
        String::from("EFGH"),
    );

    let root = state_machine.state.root();

    env::commit(&"ABCD");
}
