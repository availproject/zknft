use crate::{
    errors::Error,
    nft::types::{CallType, Nft, NftCallParams, NftId},
    state::State,
    traits::{Leaf, StateMachine},
    types::StateUpdate,
};
use primitive_types::U256;
use risc0_zkvm::sha::rust_crypto::{Digest as _};
use sparse_merkle_tree::{
    traits::Value,
};

pub struct NftStateMachine {
    pub state: State<Nft>,
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

        let mut state = State::new();

        state
            .update_set(vec![nft1, nft2])
            .expect("Init state failed.");

        NftStateMachine { state }
    }

    fn call(&mut self, params: NftCallParams) -> Result<StateUpdate<Nft>, Error> {
        match params.call_type {
            CallType::Transfer => self.transfer(params),
            CallType::Mint => self.mint(params),
            CallType::Burn => self.burn(params),
        }
    }

    fn load() -> Self {
        unimplemented!()
        // // let general_store = DB::open_default("./demo_data/2").expect("Unable to find DB.");
        // let bytes: [u8; 32] = [127, 116, 122, 219, 223, 166, 3, 8, 126, 27, 73, 169, 153, 127, 141, 212, 184, 249, 23, 184, 124, 166, 180, 187, 129, 174, 230, 85, 188, 240, 207, 115];
        // let state = MerkleTree::<NftTable, StorageType>::load(store, &bytes).unwrap();

        // StateMachine {
        //     state,
        // //    store: general_store,
        // }
    }
}

impl NftStateMachine {
    fn transfer(&mut self, params: NftCallParams) -> Result<StateUpdate<Nft>, Error> {
        let nft_key = params.id.get_key();

        let nft_to_transfer =  match self.state.get(&nft_key) {
            Ok(Some(i)) => i,
            Err(e) => return Err(e),
            Ok(None) => return Err(Error::Unknown),
        };

        if nft_to_transfer.owner != params.from {
            panic!("Not owner");
        }

        let transferred_nft = Nft {
            id: params.id,
            owner: params.owner.unwrap(),
        };

        self.state.update_set(vec![transferred_nft])
    }

    fn mint(&mut self, params: NftCallParams) -> Result<StateUpdate<Nft>, Error> {
        let nft_key = params.id.get_key();

        match self.state.get(&nft_key) {
            Ok(Some(_i)) => panic!("Already minted"),
            Err(e) => return Err(e),
            Ok(None) => (),
        }

        //TODO: Add runtime check to see if owner is mentioned.
        let nft = Nft {
            id: params.id,
            owner: match params.owner {
                Some(i) => i,
                None => String::from("Default Owner"),
            },
        };

        self.state.update_set(vec![nft])
    }

    fn burn(&mut self, params: NftCallParams) -> Result<StateUpdate<Nft>, Error> {
        let nft_key = params.id.get_key();

        let mut nft: Nft = match self.state.get(&nft_key) {
            Ok(Some(i)) => i,
            Err(_e) => panic!("Error in finding nft"),
            Ok(None) => panic!("Nft does not exist"),
        };

        if nft.owner != params.from {
            panic!("Not owner")
        }

        nft = Nft::zero();

        self.state.update_set(vec![nft])
    }
}
