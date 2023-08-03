use crate::{
    errors::Error,
    nft::types::{CallType, Nft, NftCallParams},
    traits::{Leaf, ZkVMStateMachine},
    types::{ShaHasher, StateUpdate},
};

use risc0_zkvm::sha::rust_crypto::{Digest as _};

use sparse_merkle_tree::{
    traits::Value,
};


pub struct NftStateMachine {}

impl ZkVMStateMachine<Nft> for NftStateMachine {
    type CallParams = NftCallParams;

    fn new() -> Self {
        NftStateMachine {}
    }

    fn call(&self, params: NftCallParams, state_update: StateUpdate<Nft>) -> Result<(), Error> {
        match state_update.pre_state_with_proof.1.verify::<ShaHasher>(
            &state_update.pre_state_root,
            state_update
                .pre_state_with_proof
                .0
                .iter()
                .map(|v| (v.id.get_key(), v.to_h256()))
                .collect(),
        ) {
            Ok(true) => (),
            //TODO - Change to invalid proof error
            Ok(false) => return Err(Error::Unknown),
            Err(_i) => return Err(Error::Unknown),
        };

        let call_result: Result<Nft, Error> = match params.call_type {
            CallType::Transfer => {
                self.transfer(params, state_update.pre_state_with_proof.0[0].clone())
            }
            CallType::Mint => self.mint(params, state_update.pre_state_with_proof.0[0].clone()),
            CallType::Burn => self.burn(params, state_update.pre_state_with_proof.0[0].clone()),
        };

        let updated_nft: Nft = match call_result {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match state_update.post_state_with_proof.1.verify::<ShaHasher>(
            &state_update.post_state_root,
            vec![(updated_nft.get_key(), updated_nft.to_h256())],
        ) {
            Ok(true) => (),
            //TODO - Change to invalid proof error
            Ok(false) => return Err(Error::Unknown),
            Err(_i) => return Err(Error::Unknown),
        };

        Ok(())
    }
}

impl NftStateMachine {
    fn transfer(&self, params: NftCallParams, pre_state: Nft) -> Result<Nft, Error> {
        if pre_state.owner != params.from {
            panic!("Not owner");
        }

        Ok(Nft {
            id: params.id,
            owner: params.owner.unwrap(),
        })
    }

    fn mint(&self, params: NftCallParams, pre_state: Nft) -> Result<Nft, Error> {
        if !pre_state.owner.is_empty() {
            panic!("Already minted");
        }

        //TODO: Add runtime check to see if owner is passed.
        Ok(Nft {
            id: params.id,
            owner: match params.owner {
                Some(i) => i,
                None => String::from("Default Owner"),
            },
        })
    }

    fn burn(&self, params: NftCallParams, pre_state: Nft) -> Result<Nft, Error> {
        if pre_state.owner.is_empty() {
            panic!("Nft does not exist");
        }

        if pre_state.owner != params.from {
            panic!("Not owner")
        }

        Ok(Nft::zero())
    }
}
