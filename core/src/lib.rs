pub use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[cfg(feature = "native")]
pub mod native_storage;
pub mod state_machine;
pub mod zk_storage;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct NftId(pub U256);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Nft {
    pub id: NftId,
    pub owner: String,
}

// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// pub struct Metadata {
//     pub name: String,
//     pub description: String,
//     pub image: String,
// }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserRequest {
    pub owner: String,
    pub nft_id: String,
    pub from: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Commit {
    pub pre_state_root: String,
    pub post_state_root: String,
}

// fn main() {
//     let store = native_storage::NativeStorage::from_path("./demo_data/1".to_string());
//     let mut state_machine = state_machine::StateMachine::new(store);
//     let root = state_machine.state.root();

//     println!("\n ROOT: {:?}", root);

//     state_machine.transfer(
//         String::from("EFGH"),
//         String::from("2"),
//         String::from("EFGH"),
//     );

//     let root = state_machine.state.root();

//     println!("\n ROOT: {:?}", root);
// }
