use crate::{
    errors::Error,
    payments::types::{Account, CallParams as PaymentsCallParams, CallType},
    traits::{Leaf, ZkVMStateMachine},
    types::{ShaHasher, StateUpdate},
};

use risc0_zkvm::sha::rust_crypto::Digest as _;

use sparse_merkle_tree::traits::Value;

pub struct PaymentsStateMachine {}

impl ZkVMStateMachine<Account> for PaymentsStateMachine {
    type CallParams = PaymentsCallParams;

    fn new() -> Self {
        PaymentsStateMachine {}
    }

    fn call(
        &self,
        params: PaymentsCallParams,
        state_update: StateUpdate<Account>,
    ) -> Result<(), Error> {
        match state_update.pre_state_with_proof.1.verify::<ShaHasher>(
            &state_update.pre_state_root,
            state_update
                .pre_state_with_proof
                .0
                .iter()
                .map(|v| (v.address.get_key(), v.to_h256()))
                .collect(),
        ) {
            Ok(true) => (),
            //TODO - Change to invalid proof error
            Ok(false) => return Err(Error::Unknown),
            Err(_i) => return Err(Error::Unknown),
        };

        let call_result: Result<Vec<Account>, Error> = match params.call_type {
            CallType::Transfer => {
                self.transfer(params, state_update.pre_state_with_proof.0.clone())
            }
            CallType::Mint => self.mint(params, state_update.pre_state_with_proof.0[0].clone()),
        };

        let updated_accounts: Vec<Account> = match call_result {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match state_update.post_state_with_proof.1.verify::<ShaHasher>(
            &state_update.post_state_root,
            updated_accounts
                .into_iter()
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

impl PaymentsStateMachine {
    fn transfer(
        &self,
        params: PaymentsCallParams,
        pre_state: Vec<Account>,
    ) -> Result<Vec<Account>, Error> {
        let mut from_account = pre_state[0].clone();
        let mut to_account = pre_state[1].clone();

        if from_account.balance < params.amount {
            panic!("Not enough balance");
        }

        from_account.balance -= params.amount;
        to_account.balance += params.amount;

        Ok(vec![from_account, to_account])
    }

    fn mint(&self, params: PaymentsCallParams, account: Account) -> Result<Vec<Account>, Error> {
        let mut to_account = account;

        to_account.balance += params.amount;

        Ok(vec![to_account])
    }
}
