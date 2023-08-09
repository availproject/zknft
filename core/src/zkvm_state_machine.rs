use crate::{
    errors::Error,
    nft::types::CallType,
    state::VmState,
    traits::{Leaf, StateTransition},
    types::{ShaHasher, StateUpdate},
};
use sparse_merkle_tree::{traits::Value, H256};
use std::marker::PhantomData;

pub struct ZKStateMachine<V, S: StateTransition<V>> {
    stf: S,
    phantom_v: PhantomData<V>,
}

impl<V: Leaf<H256> + Value + Clone, S: StateTransition<V>> ZKStateMachine<V, S> {
    fn new(stf: S) -> Self {
        ZKStateMachine {
            stf,
            phantom_v: PhantomData,
        }
    }

    fn call(&self, params: S::CallParams, state_update: StateUpdate<V>) -> Result<(), Error> {
        match state_update.pre_state_with_proof.1.verify::<ShaHasher>(
            &state_update.pre_state_root,
            state_update
                .pre_state_with_proof
                .0
                .iter()
                .map(|v| (v.get_key(), v.to_h256()))
                .collect(),
        ) {
            Ok(true) => (),
            //TODO - Change to invalid proof error
            Ok(false) => return Err(Error::Unknown),
            Err(_i) => return Err(Error::Unknown),
        };

        let call_result: Result<Vec<V>, Error> = self
            .stf
            .execute(state_update.pre_state_with_proof.0.clone(), params);

        let updated_set: Vec<V> = match call_result {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match state_update.post_state_with_proof.1.verify::<ShaHasher>(
            &state_update.post_state_root,
            updated_set
                .iter()
                .map(|x| (x.get_key(), x.to_h256()))
                .collect(),
        ) {
            Ok(true) => (),
            //TODO - Change to invalid proof error
            Ok(false) => return Err(Error::Unknown),
            Err(_i) => return Err(Error::Unknown),
        };

        Ok(())
    }
}
