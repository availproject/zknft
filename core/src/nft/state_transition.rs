use crate::{
    errors::Error,
    nft::types::{CallType, Nft, NftTransaction},
    traits::StateTransition,
};
use sparse_merkle_tree::traits::Value;

pub struct NftStateTransition;

impl NftStateTransition {
    pub fn new() -> Self {
        NftStateTransition {}
    }

    fn transfer(&self, params: NftTransaction, pre_state: Nft) -> Result<Vec<Nft>, Error> {
        if pre_state.owner != params.from {
            panic!("Not owner");
        }

        Ok(vec![Nft {
            id: params.id,
            owner: params.owner.unwrap(),
        }])
    }

    fn mint(&self, params: NftTransaction, pre_state: Nft) -> Result<Vec<Nft>, Error> {
        if !pre_state.owner.is_empty() {
            panic!("Already minted");
        }

        //TODO: Add runtime check to see if owner is passed.
        Ok(vec![Nft {
            id: params.id,
            owner: match params.owner {
                Some(i) => i,
                None => String::from("Default Owner"),
            },
        }])
    }

    fn burn(&self, params: NftTransaction, pre_state: Nft) -> Result<Vec<Nft>, Error> {
        if pre_state.owner.is_empty() {
            panic!("Nft does not exist");
        }

        if pre_state.owner != params.from {
            panic!("Not owner")
        }

        Ok(vec![Nft::zero()])
    }
}

impl StateTransition<Nft, NftTransaction> for NftStateTransition {
    fn execute(&self, pre_state: Vec<Nft>, params: NftTransaction) -> Result<Vec<Nft>, Error> {
        match params.call_type {
            CallType::Transfer => self.transfer(params, pre_state[0].clone()),
            CallType::Mint => self.mint(params, pre_state[0].clone()),
            CallType::Burn => self.burn(params, pre_state[0].clone()),
        }
    }
}
