#![no_main]
use nft_core::{ 
    nft::{types::{Nft, NftTransaction}, state_transition::NftStateTransition}, 
    types::{StateUpdate}, 
    zkvm_state_machine::ZKStateMachine
};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let nft_call_params: NftTransaction = env::read();
    let state_update: StateUpdate<Nft> = env::read();

    let state_machine = ZKStateMachine::new(NftStateTransition::new());

    match state_machine.call(nft_call_params, state_update.clone()) {
        Ok(()) => (), 
        Err(_) => panic!("State transition failed.")
    }
    env::commit(&state_update.post_state_root);
}
