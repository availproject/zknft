use crate::{errors::Error, types::StateUpdate};

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait StateMachine<V> {
    type CallParams;

    fn new() -> Self;
    fn load() -> Self;
    fn call(&mut self, call: Self::CallParams) -> Result<StateUpdate<V>, Error>;
}

pub trait ZkVMStateMachine<V> {
    type CallParams;

    fn new() -> Self;
    fn call(&self, call: Self::CallParams, state_update: StateUpdate<V>) -> Result<(), Error>;
}

pub trait StateTransition<V> {
    type CallParams;
    //Requiring the Value to be in a vector adds overhead when only one state is modified,
    //but we do it for sake of simplicity.
    fn execute(&self, pre_state: Vec<V>, call_params: Self::CallParams) -> Result<Vec<V>, Error>;
}
