use crate::{
    errors::Error,
    payments::state_transition::PaymentsStateTransition,
    payments::types::{
        Account, Transaction as PaymentsTransaction,
    },
    state::VmState,
    traits::{StateMachine, StateTransition},
    types::{AggregatedBatch, StateUpdate, TransactionReceipt},
};

use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::MerkleProof;
use sparse_merkle_tree::H256;

pub struct PaymentsStateMachine {
    pub state: VmState<Account>,
    stf: PaymentsStateTransition,
}

impl StateMachine<Account, PaymentsTransaction> for PaymentsStateMachine {
    fn new(root: H256) -> Self {
        let state = VmState::new(root);

        PaymentsStateMachine {
            state,
            stf: PaymentsStateTransition::new(),
        }
    }

    fn execute_tx(
        &mut self,
        params: PaymentsTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(StateUpdate<Account>, TransactionReceipt), Error> {
        let from_address_key = params.message.from.get_key();
        let to_address_key = params.message.to.get_key();

        let from_account: Account = match self.state.get(&from_address_key) {
            Ok(Some(i)) => i,
            Err(_e) => panic!("Error in finding account details"),
            Ok(None) => Account::zero(),
        };

        let to_account = match self.state.get(&to_address_key) {
            Ok(Some(i)) => i,
            Err(_e) => panic!("Error in finding account details"),
            Ok(None) => Account::zero(),
        };

        let result =
            match self
                .stf
                .execute_tx(vec![from_account, to_account], params, aggregated_proof)
            {
                Ok(i) => i,
                Err(e) => return Err(e),
            };

        match self.state.update_set(result.0) {
            Ok(i) => Ok((i, result.1)),
            Err(e) => Err(e),
        }
    }

    fn get_state_with_proof(
        &self, 
        key: &H256, 
    ) -> Result<(Account, MerkleProof), Error> {
        self.state.get_with_proof(key)
    }

    fn revert(&mut self, root: H256) -> Result<(), Error> {
        self.state = VmState::new(root);

        Ok(())
    }
}
