use crate::{
    errors::Error,
    types::{StateUpdate, TransactionReceipt, AggregatedBatch},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sparse_merkle_tree::H256;

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait StateMachine<V, T: Clone + DeserializeOwned + Serialize> {
    fn new(root: H256) -> Self;
    fn execute_tx(&mut self, call: T,aggregated_proof: AggregatedBatch) -> Result<(StateUpdate<V>, TransactionReceipt), Error>;
}

pub trait StateTransition<V, T> {
    //Requiring the Value to be in a vector adds overhead when only one state is modified,
    //but we do it for sake of simplicity.
    fn execute_tx(
        &self,
        pre_state: Vec<V>,
        call_params: T,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(Vec<V>, TransactionReceipt), Error>;
}

pub trait TxHasher {
    fn to_h256(&self) -> H256;
}
