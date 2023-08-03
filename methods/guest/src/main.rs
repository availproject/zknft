#![no_main]
use nft_core::{ nft::{zkvm_state_machine::{NftStateMachine}, types::{Nft, NftCallParams}}, types::{StateUpdate}, traits::ZkVMStateMachine };
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let nft_call_params: NftCallParams = env::read();
    let state_update: StateUpdate<Nft> = env::read();

    let state_machine = NftStateMachine::new();

    match state_machine.call(nft_call_params, state_update.clone()) {
        Ok(()) => (), 
        Err(_) => panic!("State transition failed.")
    }
    env::commit(&state_update.post_state_root);
}
