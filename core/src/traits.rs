use crate::{errors::Error, types::StateUpdate};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sparse_merkle_tree::H256;

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait StateMachine<V, T: Clone + DeserializeOwned + Serialize> {
    fn new(root: H256) -> Self;
    fn call(&mut self, call: T) -> Result<StateUpdate<V>, Error>;
}

pub trait ZkVMStateMachine<V, T> {
    fn new() -> Self;
    fn call(&self, call: T, state_update: StateUpdate<V>) -> Result<(), Error>;
}

pub trait StateTransition<V, T> {
    //Requiring the Value to be in a vector adds overhead when only one state is modified,
    //but we do it for sake of simplicity.
    fn execute(&self, pre_state: Vec<V>, call_params: T) -> Result<Vec<V>, Error>;
}

pub trait TxHasher {
    fn to_h256(&self) -> H256;
}
