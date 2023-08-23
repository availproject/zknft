use nft_core::{
    nft::{
        state_machine::NftStateMachine,
        types::{CallType, Nft, NftTransaction, NftId},
    },
    traits::StateMachine,
    app_node::{AppNode, AppNodeRuntimeConfig, start_rpc_server}
};
use nft_methods::{TRANSFER_ELF, TRANSFER_ID};
use primitive_types::U256;
use risc0_zkvm::{
    default_executor_from_elf,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};
use serde::ser::Serialize;
use sparse_merkle_tree::{
    default_store::DefaultStore, error::Error, traits::Hasher, traits::Value, MerkleProof,
    SparseMerkleTree, H256,
};
use std::time::SystemTime;

#[tokio::main]
async fn main() {
    let mut app = AppNode::<Nft, NftTransaction, NftStateMachine>::new(
        AppNodeRuntimeConfig {
            prover_mode: true
        }, 
        TRANSFER_ELF, 
        TRANSFER_ID
    );

    
    start_rpc_server(app).await;
    ()
}
