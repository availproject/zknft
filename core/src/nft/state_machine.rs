use crate::{
    errors::Error,
    nft::types::{CallType, Nft, NftCallParams, NftId},
    nft::state_transition::NftStateTransition,
    state::VmState,
    traits::StateMachine,
    types::StateUpdate,
};
use primitive_types::U256;
use crate::traits::StateTransition;
use sparse_merkle_tree::traits::Value;

pub struct NftStateMachine {
    pub state: VmState<Nft>,
    stf: NftStateTransition
}

impl StateMachine<Nft> for NftStateMachine {
    type CallParams = NftCallParams;

    fn new() -> Self {
        // let general_store = DB::open_default("./demo_data/2").expect("Unable to find DB.");

        //Code to add NFT
        let nft1 = Nft {
            id: NftId(U256::from_dec_str("1").unwrap()),
            owner: String::from("ABCD"),
        };
        let nft2 = Nft {
            id: NftId(U256::from_dec_str("2").unwrap()),
            owner: String::from("EFGH"),
        };

        let mut state = VmState::new();

        state
            .update_set(vec![nft1, nft2])
            .expect("Init state failed.");

        NftStateMachine { 
            state, 
            stf: NftStateTransition::new() 
        }
    }

    fn call(&mut self, params: NftCallParams) -> Result<StateUpdate<Nft>, Error> {
        let nft_key = params.id.get_key();
        let nft = match self.state.get(&nft_key) {
            Ok(Some(i)) => i,
            Err(e) => return Err(e),
            Ok(None) => return Err(Error::Unknown),
        };

        let updated_set = match self.stf.execute(vec![nft], params){
            Ok(i) => i, 
            Err(e) => return Err(e)
        };

        self.state.update_set(updated_set)
    }

    fn load() -> Self {
        unimplemented!()
    }
}
