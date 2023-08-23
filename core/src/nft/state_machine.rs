use crate::traits::StateTransition;
use crate::{
    errors::Error,
    nft::state_transition::NftStateTransition,
    nft::types::{CallType, Nft, NftTransaction, NftId},
    state::VmState,
    traits::StateMachine,
    types::StateUpdate,
};
use primitive_types::U256;
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;

pub struct NftStateMachine {
    pub state: VmState<Nft>,
    stf: NftStateTransition,
}

impl StateMachine<Nft, NftTransaction> for NftStateMachine {
    fn new(root: H256) -> Self {
        let mut state = VmState::new(root);

        //TODO: Can remove get root here.
        if state.get_root() == H256::zero() {
            //Init state if not done previously.
            let nft1 = Nft {
                id: NftId(U256::from_dec_str("1").unwrap()),
                owner: String::from("ABCD"),
            };
            let nft2 = Nft {
                id: NftId(U256::from_dec_str("2").unwrap()),
                owner: String::from("EFGH"),
            };

            state
                .update_set(vec![nft1, nft2])
                .expect("Init state failed.");
        }

        NftStateMachine {
            state,
            stf: NftStateTransition::new(),
        }
    }

    fn call(&mut self, params: NftTransaction) -> Result<StateUpdate<Nft>, Error> {
        let nft_key = params.id.get_key();
        let nft = match self.state.get(&nft_key) {
            Ok(Some(i)) => i,
            Err(e) => return Err(e),
            Ok(None) => return Err(Error::Unknown),
        };

        let updated_set = match self.stf.execute(vec![nft], params) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        self.state.update_set(updated_set)
    }
}
