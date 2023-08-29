use payments_methods::{TRANSFER_ELF, TRANSFER_ID};
use nft_core::{
    payments::{
        state_machine::PaymentsStateMachine, 
        types::{Account, Address, CallType, Transaction}
    },
    traits::StateMachine,
    app_node::{AppNode, AppNodeRuntimeConfig, start_rpc_server, AppChain}
};
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
    let mut app = AppNode::<Account, Transaction, PaymentsStateMachine>::new(
        AppNodeRuntimeConfig {
            prover_mode: true
        }, 
        TRANSFER_ELF, 
        TRANSFER_ID, 
        AppChain::Payments
    );

    println!("{:?}", TRANSFER_ID);
    start_rpc_server(app).await;
    ()
}
