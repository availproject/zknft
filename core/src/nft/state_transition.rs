use crate::{
    errors::Error,
    nft::types::{
        Burn, Future, FutureReceiptData, Mint, Nft, NftTransaction, Transfer, TransferReceiptData,
        Trigger,
    },
    traits::StateTransition,
    types::{AggregatedBatch, ShaHasher, TransactionReceipt},
};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;

pub struct NftStateTransition;

impl NftStateTransition {
    pub fn new() -> Self {
        NftStateTransition {}
    }

    fn transfer(
        &self,
        params: Transfer,
        pre_state: Nft,
    ) -> Result<(Vec<Nft>, TransactionReceipt), Error> {
        if pre_state == Nft::zero() {
            panic!("NFT not minted.");
        }

        if pre_state.owner != params.from {
            panic!("Not owner");
        }

        let updated_nonce = pre_state.nonce + 1;

        match params.future_commitment {
            None => Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: params.to.clone(),
                    nonce: updated_nonce,
                    future: None,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (TransferReceiptData {
                        id: params.id,
                        from: params.from,
                        to: params.to,
                        data: params.data,
                        nonce: updated_nonce,
                    })
                    .to_vec(),
                },
            )),
            Some(i) => Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: pre_state.owner.clone(),
                    future: Some(Future {
                        to: params.to.clone(),
                        commitment: i,
                    }),
                    nonce: updated_nonce,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (FutureReceiptData {
                        id: params.id,
                        from: params.from,
                        to: params.to,
                        data: params.data,
                        nonce: updated_nonce,
                        future_commitment: i,
                    })
                    .to_vec(),
                },
            )),
        }
    }

    fn mint(&self, params: Mint, pre_state: Nft) -> Result<(Vec<Nft>, TransactionReceipt), Error> {
        if pre_state != Nft::zero() {
            panic!("Already minted");
        }

        match params.future_commitment {
            None => Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: params.to.clone(),
                    nonce: 1,
                    future: None,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (TransferReceiptData {
                        id: params.id,
                        from: String::from(""),
                        to: params.to,
                        data: params.data,
                        nonce: 1,
                    })
                    .to_vec(),
                },
            )),
            Some(i) => Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: String::from(""),
                    nonce: 1,
                    future: Some(Future {
                        to: params.to.clone(),
                        commitment: i,
                    }),
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (FutureReceiptData {
                        id: params.id,
                        from: String::from(""),
                        to: params.to,
                        data: params.data,
                        nonce: 1,
                        future_commitment: i,
                    })
                    .to_vec(),
                },
            )),
        }
    }

    fn burn(&self, params: Burn, pre_state: Nft) -> Result<(Vec<Nft>, TransactionReceipt), Error> {
        if pre_state == Nft::zero() {
            panic!("Nft does not exist");
        }

        if pre_state.owner != params.from {
            panic!("Not owner")
        }

        let updated_nonce = pre_state.nonce + 1;

        match params.future_commitment {
            None => Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: String::from(""),
                    nonce: updated_nonce,
                    future: None,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (TransferReceiptData {
                        id: params.id,
                        from: params.from,
                        to: String::from(""),
                        data: params.data,
                        nonce: updated_nonce,
                    })
                    .to_vec(),
                },
            )),
            Some(i) => Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: pre_state.owner.clone(),
                    future: Some(Future {
                        to: String::from(""),
                        commitment: i,
                    }),
                    nonce: updated_nonce,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (FutureReceiptData {
                        id: params.id,
                        from: params.from,
                        to: String::from(""),
                        data: params.data,
                        nonce: updated_nonce,
                        future_commitment: i,
                    })
                    .to_vec(),
                },
            )),
        }
    }

    fn trigger(
        &self,
        params: Trigger,
        pre_state: Nft,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(Vec<Nft>, TransactionReceipt), Error> {
        if pre_state == Nft::zero() {
            panic!("Nft does not exist.");
        }

        let future = match pre_state.future {
            None => panic!("No future registered."),
            Some(i) => i,
        };

        match params.merkle_proof.verify::<ShaHasher>(
            &aggregated_proof.receipts_root,
            //Checks both inclusion or non inclusion.
            vec![(future.commitment.clone(), params.receipt.to_h256())],
        ) {
            Ok(true) => (),
            Ok(false) => panic!("Invalid merkle proof."),
            Err(e) => panic!("Error while verifying merkle"),
        }

        let updated_nonce = pre_state.nonce + 1;

        //Check if the given proof was non inclusion.
        if params.receipt.to_h256() == H256::zero() {
            //Revert transaction if the receipt is not included.
            Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: pre_state.owner.clone(),
                    future: None,
                    nonce: updated_nonce,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (TransferReceiptData {
                        id: params.id,
                        from: pre_state.owner.clone(),
                        to: pre_state.owner,
                        data: params.data,
                        nonce: updated_nonce,
                    })
                    .to_vec(),
                },
            ))
        } else {
            Ok((
                vec![Nft {
                    id: params.id.clone(),
                    owner: future.to.clone(),
                    future: None,
                    nonce: updated_nonce,
                }],
                TransactionReceipt {
                    chain_id: 7000,
                    data: (TransferReceiptData {
                        id: params.id,
                        from: pre_state.owner,
                        to: future.to,
                        data: params.data,
                        nonce: updated_nonce,
                    })
                    .to_vec(),
                },
            ))
        }
    }
}

impl StateTransition<Nft, NftTransaction> for NftStateTransition {
    fn execute_tx(
        &self,
        pre_state: Vec<Nft>,
        params: NftTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(Vec<Nft>, TransactionReceipt), Error> {
        match params {
            NftTransaction::Transfer(i) => self.transfer(i, pre_state[0].clone()),
            NftTransaction::Mint(i) => self.mint(i, pre_state[0].clone()),
            NftTransaction::Burn(i) => self.burn(i, pre_state[0].clone()),
            NftTransaction::Trigger(i) => self.trigger(i, pre_state[0].clone(), aggregated_proof),
        }
    }
}
