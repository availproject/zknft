use crate::{
    payments::state_transition::PaymentsStateTransition,
    payments::types::{
        Account, Transaction as PaymentsTransaction, TransactionMessage
    },
    state::VmState,
    traits::{StateMachine, StateTransition},
    types::{AggregatedBatch, StateUpdate, TransactionReceipt},
};

use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::MerkleProof;
use sparse_merkle_tree::H256;
use anyhow::{Error, anyhow};

pub struct PaymentsStateMachine {
    pub state: Option<VmState<Account>>,
    stf: PaymentsStateTransition,
}

impl StateMachine<Account, PaymentsTransaction> for PaymentsStateMachine {
    fn new(root: H256) -> Self {
        let state = VmState::new(root);

        PaymentsStateMachine {
            state: Some(state),
            stf: PaymentsStateTransition::new(),
        }
    }

    fn execute_tx(
        &mut self,
        params: PaymentsTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(StateUpdate<Account>, TransactionReceipt), Error> {
        let state = match &mut self.state {
            None => return Err(anyhow!("Internal error, restart node.")), 
            Some(i) => i,
        };

        let message: TransactionMessage = TransactionMessage::try_from(params.clone())?;
        let from_address_key = message.from.get_key();
        let to_address_key = message.to.get_key();

        let from_account: Account = match state.get(&from_address_key) {
            Ok(Some(i)) => i,
            Err(_e) => return Err(anyhow!("Error in finding account details")),
            Ok(None) => Account::zero(),
        };

        let to_account = match state.get(&to_address_key) {
            Ok(Some(i)) => i,
            Err(_e) => return Err(anyhow!("Error in finding account details")),
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

        match state.update_set(result.0) {
            Ok(i) => Ok((i, result.1)),
            Err(e) => Err(e),
        }
    }

    fn get_state_with_proof(
        &self, 
        key: &H256, 
    ) -> Result<(Account, MerkleProof), Error> {
        let state = match &self.state {
            None => return Err(anyhow!("Internal error, restart node.")), 
            Some(i) => i,
        };

        state.get_with_proof(key)
    }

    fn revert(&mut self, root: H256) -> Result<(), Error> {
        let old_state = std::mem::take(&mut self.state);

        // Check if there was a previous state to work with
        if let Some(old_state) = old_state {
            // Perform the revert operation on the old state
            let new_state = old_state.revert(root);
            
            // Assign the new state back to self.state
            self.state = Some(new_state);
        } else {
            // Handle the case where there was no old state
            return Err(anyhow!("No previous state available. Need to restart."));
        }

        let state = match &self.state {
            None => return Err(anyhow!("Revert failed. Restart node.")), 
            Some(i) => i,
        };

        println!("Reverted state machine to root: {:?}", &state.get_root());

        Ok(())
    }

    fn get_root(&self) -> Result<H256, Error> {
        match &self.state {
            Some(i) => Ok(i.get_root()), 
            None => Err(anyhow!("Critical error, tree state missing.")),
        }
    }
}
