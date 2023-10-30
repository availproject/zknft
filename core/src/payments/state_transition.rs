use crate::traits::StateTransition;
use crate::{
    payments::types::{
        Account, CallType, PaymentReceiptData, Transaction as PaymentsTransaction, TransactionMessage
    },
    traits::StateMachine,
    types::{AggregatedBatch, TransactionReceipt, Address},
};
use sparse_merkle_tree::traits::Value;

use anyhow::{Error, anyhow};

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
        #[cfg(any(feature = "native", feature = "native-metal"))]
        println!("\n Executing following transaction: {:?} \n", &params);

        let mut from_account: Account = match pre_state[0].clone() {
            i if i == Account::zero() => Account {
                address: params.from.clone(),
                nonce: 0,
                balance: 0,
            },
            i => i,
        };

        println!("{:?}", from_account);

        if from_account.balance < params.amount {
            return Err(anyhow!("not enough balance."))
        }

        if from_account.address == params.to {
            return Err(anyhow!("Cannot transfer to self."));
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
                .to_encoded(),
            },
        ))
    }

    fn mint(
        &self,
        params: TransactionMessage,
        pre_state: Vec<Account>,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        #[cfg(any(feature = "native", feature = "native-metal"))]
        println!("\n Executing following transaction: {:?} \n", &params);

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


            #[cfg(any(feature = "native", feature = "native-metal"))]
            println!("Transaction state update: {:?}", vec![from_account.clone(), to_account.clone()]);

            Ok((
                vec![from_account.clone(), to_account],
                TransactionReceipt {
                    chain_id: self.chain_id,
                    data: (PaymentReceiptData {
                        from: Address([0u8; 32]),
                        to: params.to,
                        amount: params.amount,
                        call_type: params.call_type,
                        data: params.data,
                        nonce: from_account.nonce,
                    })
                    .to_encoded(),
                },
            ))
        } else {
            from_account.balance += params.amount;

            Ok((
                vec![from_account.clone()],
                TransactionReceipt {
                    chain_id: self.chain_id,
                    data: (PaymentReceiptData {
                        from: Address([0u8; 32]),
                        to: params.to,
                        amount: params.amount,
                        call_type: params.call_type,
                        data: params.data,
                        nonce: from_account.nonce,
                    })
                    .to_encoded(),
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
        _aggregated_proof: AggregatedBatch,
    ) -> Result<(Vec<Account>, TransactionReceipt), Error> {
        let message: TransactionMessage = TransactionMessage::try_from(params.clone())?;
        match message.from.verify_msg(&params.signature, &params.message) {
            true => (), 
            false => return Err(anyhow!("Signature verification failed.")),
        }

        match message.call_type {
            CallType::Transfer => self.transfer(message, pre_state),
            CallType::Mint => self.mint(message, pre_state),
        }
    }
}
