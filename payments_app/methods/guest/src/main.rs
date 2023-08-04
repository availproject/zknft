#![no_main]
use nft_core::{     
    payments::{
        zkvm_state_machine::PaymentsStateMachine, 
        types::{Account, Address, CallType, CallParams}
    },
    types::{StateUpdate}, 
    traits::ZkVMStateMachine 
};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let payments_call_params: CallParams = env::read();
    let state_update: StateUpdate<Account> = env::read();

    let state_machine = PaymentsStateMachine::new();

    match state_machine.call(payments_call_params, state_update.clone()) {
        Ok(()) => (), 
        Err(_) => panic!("State transition failed.")
    }
    env::commit(&state_update.post_state_root);
}
