use crate::db::NodeDB;
use crate::traits::StateTransition;
use crate::{
    nft::state_transition::NftStateTransition,
    nft::types::{Nft, NftId, NftTransaction, NftTransactionMessage},
    state::VmState,
    traits::StateMachine,
    types::{Address, AggregatedBatch, StateUpdate, TransactionReceipt},
};
use anyhow::{anyhow, Error};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::MerkleProof;
use sparse_merkle_tree::H256;

pub struct NftStateMachine {
    state: VmState<Nft>,
    stf: NftStateTransition,
    custodian: Option<Address>,
    db: NodeDB,
}

impl NftStateMachine {
    pub fn register_custodian(&mut self, address: Address) {
        println!("Registering custodian: {:?}", &address);
        self.custodian = Some(address);
    }

    pub fn get_listed_nfts(&self) -> Result<Vec<Nft>, Error> {
        let listed_nft_ids: Vec<NftId> = match self.db.get(b"all_listed_nfts") {
            Ok(Some(i)) => i,
            Ok(None) => vec![],
            Err(e) => return Err(anyhow!("Could not access db due to error: . {:?}", e)),
        };

        println!("Listed nft ids: {:?}", &listed_nft_ids);

        let mut listed_nfts: Vec<Nft> = vec![];

        //Get latest state of all listed nfts.
        for id in &listed_nft_ids {
            match self.state.get(&id.get_key()) {
                Ok(Some(i)) => listed_nfts.push(i),
                Ok(None) => (),
                Err(e) => return Err(anyhow!("Could not get nft from db: {:?}", e)),
            }
        }

        println!("Listed nft ids: {:?}", &listed_nft_ids);

        Ok(listed_nfts)
    }
}

impl StateMachine<Nft, NftTransaction> for NftStateMachine {
    fn new(root: H256) -> Self {
        let state = VmState::new(root);
        let node_db = NodeDB::from_path(String::from("./marketplace_db"));

        NftStateMachine {
            state: state,
            stf: NftStateTransition::new(),
            custodian: None,
            db: node_db,
        }
    }

    fn execute_tx(
        &mut self,
        params: NftTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<(StateUpdate<Nft>, TransactionReceipt), Error> {
        let message: NftTransactionMessage = NftTransactionMessage::try_from(params.clone())?;

        let nft_id = match message {
            NftTransactionMessage::Transfer(ref i) => i.id.clone(),
            NftTransactionMessage::Mint(ref i) => i.id.clone(),
            NftTransactionMessage::Burn(ref i) => i.id.clone(),
            NftTransactionMessage::Trigger(ref i) => i.id.clone(),
        };
        let nft_key = nft_id.get_key();

        println!("{:?}", &nft_key);

        let nft = match self.state.get(&nft_key) {
            Ok(Some(i)) => i,
            Err(e) => return Err(e),
            Ok(None) => Nft::zero(),
        };

        let result = match self
            .stf
            .execute_tx(vec![nft.clone()], params, aggregated_proof)
        {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        let updated_set = result.0;

        let (update, receipt) = match self.state.update_set(updated_set.clone()) {
            Ok(i) => (i, result.1),
            Err(e) => return Err(e),
        };

        match &self.custodian {
            Some(custodian) => {
                let listed_nfts: Vec<NftId> = updated_set
                    .clone()
                    .into_iter()
                    .filter(|i| i.owner == custodian.clone())
                    .map(|i| i.id)
                    .collect();

                let unlisted_nfts: Vec<NftId> = {
                    if nft.owner == custodian.clone() {
                        let updated_nft: Nft = updated_set
                            .iter()
                            .filter(|i| i.id == nft.id)
                            .collect::<Vec<&Nft>>()[0]
                            .clone(); //Only one nft will have the same id.

                        if updated_nft.owner != custodian.clone() {
                            //Returning the nft, as it is no longer listed under marketplace custodian.
                            vec![nft.id]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                };

                let mut all_listed_nfts: Vec<NftId> = match self.db.get(b"all_listed_nfts") {
                    Ok(Some(i)) => i,
                    Ok(None) => vec![],
                    Err(e) => return Err(anyhow!("Could not access db due to error: . {:?}", e)),
                };

                // Add listed_nfts to all_listed_nfts if they don't already exist
                for listed_nft in &listed_nfts {
                    if !all_listed_nfts.iter().any(|nft| nft == listed_nft) {
                        all_listed_nfts.push(listed_nft.clone());
                    }
                }

                // Remove unlisted_nfts from all_listed_nfts
                all_listed_nfts.retain(|nft| !unlisted_nfts.iter().any(|unlisted| unlisted == nft));

                self.db.put(b"all_listed_nfts", &all_listed_nfts)?;
            }
            None => (),
        }

        Ok((update, receipt))
    }

    fn get_state_with_proof(&self, key: &H256) -> Result<(Nft, MerkleProof), Error> {
        self.state.get_with_proof(key)
    }

    fn revert(&mut self) -> Result<(), Error> {
        self.state.revert()
    }

    fn commit(&mut self) -> Result<(), Error> {
        self.state.commit()
    }

    fn get_root(&self) -> Result<H256, Error> {
        Ok(self.state.get_root())
    }
}
