#![no_main]
use nft_core::{     
    payments::{
        types::{Account, CallType, Transaction},
        state_transition::PaymentsStateTransition
    },
    types::{StateUpdate, AggregatedBatch}, 
    zkvm_state_machine::ZKStateMachine
};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let payments_call_params: Transaction = env::read();
    let state_update: StateUpdate<Account> = env::read();
    let batch_number: u64 = env::read();
    let aggregated_proof: AggregatedBatch = env::read();
    let state_machine = ZKStateMachine::new(PaymentsStateTransition::new());

    let journal = match state_machine.execute_tx(payments_call_params, state_update.clone(), batch_number, aggregated_proof) {
        Ok(i) => i,
        Err(e) => {
            println!("{:?}", e);
            panic!("State transition failed.")
        }
    };

    env::commit(&journal);
}
