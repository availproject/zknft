use crate::traits::StateTransition;
use crate::{
    errors::Error,
    payments::types::{
        Account, CallType, PaymentReceiptData, Transaction as PaymentsTransaction, TransactionMessage
    },
    traits::StateMachine,
    types::{AggregatedBatch, StateUpdate, TransactionReceipt, Address},
};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;
use ed25519_dalek::SignatureError;

pub struct PaymentsStateTransition {
    chain_id: u64,
}

impl PaymentsStateTransition {
    pub fn new() -> Self {
        PaymentsStateTransition {
            //TODO: make chain ID configurable.
            chain_id: 7001,
        }
    }

    fn transfer(
        &self,
        params: TransactionMessage,
        pre_state: Vec<Account>,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        let mut from_account: Account = match pre_state[0].clone() {
            i if i == Account::zero() => Account {
                address: params.from.clone(),
                nonce: 0,
                balance: 0,
            },
            i => i,
        };

        if from_account.balance < params.amount {
            panic!("Not enough balance");
        }

        if from_account.address == params.to {
            panic!("Cannot transfer to self.");
        }

        from_account.balance -= params.amount;
        from_account.nonce += 1;

        let mut to_account: Account = match pre_state[1].clone() {
            i if i == Account::zero() => Account {
                address: params.to.clone(),
                nonce: 0,
                balance: 0,
            },
            i => i,
        };
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
        params: TransactionMessage,
        pre_state: Vec<Account>,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        let mut from_account: Account = match pre_state[0].clone() {
            i if i == Account::zero() => Account {
                address: params.from.clone(),
                nonce: 0,
                balance: 0,
            },
            i => i,
        };

        from_account.nonce += 1;

        if params.from != params.to {
            let mut to_account: Account = match pre_state[1].clone() {
                i if i == Account::zero() => Account {
                    address: params.to.clone(),
                    nonce: 0,
                    balance: 0,
                },
                i => i,
            };
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
        } else {
            from_account.balance += params.amount;

            Ok((
                vec![from_account.clone()],
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
}

impl StateTransition<Account, PaymentsTransaction> for PaymentsStateTransition {
    fn execute_tx(
        &self,
        pre_state: Vec<Account>,
        params: PaymentsTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        match params.message.from.verify_msg(&params.signature, &params.message.to_vec(), ) {
            true => (), 
            false => return Err(Error::StateTransition(String::from("Signature verification."))),
        }

        match params.message.call_type {
            CallType::Transfer => self.transfer(params.message, pre_state.clone()),
            CallType::Mint => self.mint(params.message, pre_state.clone()),
        }
    }
}
