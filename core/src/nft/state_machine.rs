use crate::traits::StateTransition;
use crate::{
    errors::Error,
    nft::state_transition::NftStateTransition,
    nft::types::{Nft, NftId, NftTransaction, NftTransactionMessage},
    state::VmState,
    traits::StateMachine,
    types::{AggregatedBatch, StateUpdate, TransactionReceipt},
};
use primitive_types::U256;
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::MerkleProof;
use sparse_merkle_tree::H256;

pub struct NftStateMachine {
    pub state: VmState<Nft>,
    stf: NftStateTransition,
}

impl StateMachine<Nft, NftTransaction> for NftStateMachine {
    fn new(root: H256) -> Self {
        let mut state = VmState::new(root);

        // TODO: Add below commented code to init, so pre_state_root matches.
        // if state.get_root() == H256::zero() {
        //     //Init state if not done previously.
        //     let nft1 = Nft {
        //         id: NftId(U256::from_dec_str("1").unwrap()),
        //         owner: String::from("ABCD"),
        //         nonce: 1,
        //         future: None,
        //     };
        //     let nft2 = Nft {
        //         id: NftId(U256::from_dec_str("2").unwrap()),
        //         owner: String::from("EFGH"),
        //         nonce: 1,
        //         future: None,
        //     };

        //     state
        //         .update_set(vec![nft1, nft2])
        //         .expect("Init state failed.");
        // }

        NftStateMachine {
            state,
            stf: NftStateTransition::new(),
        }
    }

    fn execute_tx(
        &mut self,
        params: NftTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(StateUpdate<Nft>, TransactionReceipt), Error> {
        let nft_id = match params.message {
            NftTransactionMessage::Transfer(ref i) => i.id.clone(),
            NftTransactionMessage::Mint(ref i) => i.id.clone(),
            NftTransactionMessage::Burn(ref i) => i.id.clone(),
            NftTransactionMessage::Trigger(ref i) => i.id.clone(),
        };
        let nft_key = nft_id.get_key();

        let nft = match self.state.get(&nft_key) {
            Ok(Some(i)) => i,
            Err(e) => return Err(e),
            Ok(None) => Nft::zero(),
        };

        let result = match self.stf.execute_tx(vec![nft], params, aggregated_proof) {
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
    ) -> Result<(Nft, MerkleProof), Error> {
        self.state.get_with_proof(key)
    }

    fn revert(&mut self, root: H256) -> Result<(), Error> {
        self.state = VmState::new(root);

        Ok(())
    }
}
