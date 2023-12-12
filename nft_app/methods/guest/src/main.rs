#![no_main]
use nft_core::{ 
    nft::{types::{Nft, NftTransaction}, state_transition::NftStateTransition}, 
    types::{StateUpdate, AggregatedBatch}, 
    zkvm_state_machine::ZKStateMachine
};
use risc0_zkvm::guest::env;
use nft_core::nft::types::NftTransaction;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let nft_call_params: NftTransaction = env::read();
    let state_update: StateUpdate<Nft> = env::read();
    let batch_number: u64 = env::read();
    let aggregated_proof: AggregatedBatch = env::read();
    let state_machine = ZKStateMachine::new(NftStateTransition::new());

    let journal = match state_machine.execute_tx(nft_call_params, state_update.clone(), batch_number, aggregated_proof) {
        Ok(i) => i, 
        Err(e) => {
            println!("{:?}", e);
            panic!("State transition failed.")
        }
    };

    env::commit(&journal);
}
