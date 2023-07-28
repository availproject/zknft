// TODO: Update the name of the method loaded by the prover. E.g., if the method
// is `multiply`, replace `METHOD_NAME_ELF` with `MULTIPLY_ELF` and replace
// `METHOD_NAME_ID` with `MULTIPLY_ID`
use methods::{TRANSFER_ELF, TRANSFER_ID};
use risc0_zkvm::{
    serde::{from_slice, to_vec},
    Executor, ExecutorEnv,
};
use nft_core::{ Nft, UserRequest, Commit, NftId, native_storage::NativeStorage, state_machine::StateMachine, zk_storage::ZkStorage };
use serde::ser::Serialize;
use std::time::SystemTime;
pub use primitive_types::U256;

fn main() {
    let store = NativeStorage::from_path("./demo_data/1".to_string());

    // state_machine.transfer(
    //     String::from("ABCD"),
    //     String::from("2"),
    //     String::from("ABCD"),
    // );

    // let root = state_machine.state.root();
    
    let now = SystemTime::now();
    let env = ExecutorEnv::builder()
    .add_input(&to_vec(&ZkStorage::with_db(store.db_asref())).unwrap())
    .build();

    let mut state_machine = StateMachine::new(store);
    let root = state_machine.state.root();
    println!("\n ROOT: {:?}", root);

    // Next, we make an executor, loading the (renamed) ELF binary.
    let mut exec = Executor::from_elf(env, TRANSFER_ELF).unwrap();
    // Run the executor to produce a session.
    let session = exec.run().unwrap();
    let segments = session.resolve().unwrap();

    let cycles = segments
    .iter()
    .fold(0, |acc, segment| acc + (1 << segment.po2));

    println!("Executed, cycles: {}k", cycles / 1024 );
    // Prove the session to produce a receipt.
    let receipt = session.prove().unwrap();


    // let journal: Nft = receipt.get_journal().unwrap();

    match now.elapsed() {
        Ok(elapsed) => {
            // it prints '2'
            println!("execution done, time elapsed: {}", elapsed.as_secs());
        }
        Err(e) => {
            // an error occurred!
            println!("Error: {e:?}");
        }
    }

   // println!("{:#?}", from_slice::<Commit, u8>(&receipt.journal).unwrap());

    // TODO: Implement code for transmitting or serializing the receipt for
    // other parties to verify here

    // Optional: Verify receipt to confirm that recipients will also be able to
    // verify your receipt
    //println!("{:#?}", TRANSFER_ID);
    receipt.verify(TRANSFER_ID).unwrap();

    println!("\n ROOT: {:?}", root);
}
