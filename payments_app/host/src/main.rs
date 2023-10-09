use nft_core::{
    app_node::{start_rpc_server, AppChain, AppNode, AppNodeRuntimeConfig},
    payments::{
        state_machine::PaymentsStateMachine,
        types::{Account, CallType, Transaction},
    },
    traits::StateMachine,
};
use payments_methods::{TRANSFER_ELF, TRANSFER_ID};
use primitive_types::U256;
use risc0_zkvm::{
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
        AppNodeRuntimeConfig { prover_mode: true },
        TRANSFER_ELF,
        TRANSFER_ID,
        AppChain::Payments,
    );
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let mut app_clone = app.clone();
    rt.block_on(async move {
        tokio::spawn(async move { app.run().await });

        start_rpc_server(app_clone, 7001).await;
    });
    
    ()
}
