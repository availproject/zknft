use crate::traits::StateTransition;
use crate::{
    errors::Error,
    payments::types::{
        Account, Address, CallType, PaymentReceiptData, Transaction as PaymentsTransaction,
    },
    traits::StateMachine,
    types::{StateUpdate, TransactionReceipt},
};
use sparse_merkle_tree::H256;

pub struct PaymentsStateTransition {
    chain_id: u64,
}

impl PaymentsStateTransition {
    pub fn new() -> Self {
        PaymentsStateTransition {
            //TODO: make chain ID configurable.
            chain_id: 100,
        }
    }

    fn transfer(
        &self,
        params: PaymentsTransaction,
        pre_state: Vec<Account>,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        let mut from_account = pre_state[0].clone();

        if from_account.balance < params.amount {
            panic!("Not enough balance");
        }

        from_account.balance -= params.amount;
        from_account.nonce += 1;

        let mut to_account = pre_state[1].clone();

        to_account.balance += params.amount;

        Ok((
            vec![from_account.clone(), to_account],
            TransactionReceipt {
                chain_id: self.chain_id,
                data: (PaymentReceiptData {
                    from: params.from,
                    to: params.to,
                    amount: params.amount,
                    call_type: params.call_type,
                    data: params.data,
                    nonce: from_account.nonce,
                })
                .to_vec(),
            },
        ))
    }

    fn mint(
        &self,
        params: PaymentsTransaction,
        pre_state: Vec<Account>,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        let mut from_account = pre_state[0].clone();
        let mut to_account = pre_state[1].clone();

        from_account.nonce += 1;
        to_account.balance += params.amount;

        Ok((
            vec![from_account.clone(), to_account],
            TransactionReceipt {
                chain_id: self.chain_id,
                data: (PaymentReceiptData {
                    from: Address(H256::from([0u8; 32])),
                    to: params.to,
                    amount: params.amount,
                    call_type: params.call_type,
                    data: params.data,
                    nonce: from_account.nonce,
                })
                .to_vec(),
            },
        ))
    }
}

impl StateTransition<Account, PaymentsTransaction> for PaymentsStateTransition {
    fn execute_tx(
        &self,
        pre_state: Vec<Account>,
        params: PaymentsTransaction,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        match params.call_type {
            CallType::Transfer => self.transfer(params, pre_state.clone()),
            CallType::Mint => self.mint(params, pre_state.clone()),
        }
    }
}
