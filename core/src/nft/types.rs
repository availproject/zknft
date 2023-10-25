use crate::{
    traits::{Leaf, TxHasher},
    types::{ShaHasher, TransactionReceipt, TxSignature, Address},
};
use risc0_zkvm::sha::rust_crypto::Digest;
use parity_scale_codec::{Encode, Decode};
use serde::{Deserialize, Serialize};
use ed25519_consensus::Signature;
use sparse_merkle_tree::{
    merkle_proof::MerkleProof,
    traits::{Hasher, Value},
    H256,
};
use anyhow::anyhow;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct NftId(pub [u8; 32]);

impl NftId {
    pub fn get_key(&self) -> H256 {
        H256::from(self.0)
    }
}

impl Leaf<H256> for Nft {
    fn get_key(&self) -> H256 {
        self.id.get_key()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct Nft {
    pub id: NftId,
    pub owner: Address,
    pub future: Option<Future>,
    pub nonce: u64,
    pub metadata: NftMetadata,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct NftMetadata {
    pub url: String,
    pub description: String, 
    pub name: String,
}

impl Nft {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl Value for Nft {
    fn to_h256(&self) -> H256 {
        if self.owner.is_empty() {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let encoded = self.to_encoded();
        hasher.0.update(&encoded);

        hasher.finish()
    }

    fn zero() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct Future {
    pub to: Address,
    pub commitment: H256,
}

// impl Future {
//     fn to_h256(&self) -> H256 {
//         let mut hasher = ShaHasher::new();
//         let serialized = bincode::serialize(&self.0).unwrap();
//         hasher.0.update(&serialized);

//         hasher.finish()
//     }
// }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct Transfer {
    pub id: NftId,
    pub to: Address,
    //All from to be replaced by signatures
    pub from: Address,
    pub data: Option<String>,
    pub future_commitment: Option<H256>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct Mint {
    pub id: NftId,
    pub from: Address,
    pub to: Address,
    pub data: Option<String>,
    pub future_commitment: Option<H256>,
    pub metadata: NftMetadata,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct Burn {
    pub id: NftId,
    pub from: Address,
    pub data: Option<String>,
    pub future_commitment: Option<H256>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct Trigger {
    pub id: NftId,
    pub from: Address,
    pub data: Option<String>,
    pub merkle_proof: MerkleProof,
    pub receipt: TransactionReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub enum NftTransactionMessage {
    Transfer(Transfer),
    Mint(Mint),
    Burn(Burn),
    Trigger(Trigger),
}

//TODO: Check the implications of decoding message inside ZKVM.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct NftTransaction {
    pub message: Vec<u8>, 
    pub signature: TxSignature,
}

impl NftTransaction {
    pub fn signature(&self) -> Signature {
        Signature::from(*self.signature.as_bytes())
    }

    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl NftTransactionMessage {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl TryFrom<NftTransaction> for NftTransactionMessage {
    type Error = anyhow::Error;

    fn try_from(value: NftTransaction) -> Result<Self, Self::Error> {
        let mut vec_u8 = value.message.clone();
        let mut slice_u8: &[u8] = &vec_u8;

        match NftTransactionMessage::decode(&mut slice_u8) {
            Ok(i) => Ok(i),
            Err(e) => Err(anyhow!("{:?}", e))
        }
    }
}

impl TxHasher for NftTransaction {
    fn to_h256(&self) -> H256 {
        let mut hasher = ShaHasher::new();
        let serialized = self.to_encoded();
        hasher.0.update(&serialized);

        hasher.finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct FutureReceiptData {
    pub id: NftId,
    pub from: Address,
    pub to: Address,
    pub future_commitment: H256,
    pub data: Option<String>,
    pub nonce: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct TransferReceiptData {
    pub id: NftId,
    pub from: Address,
    pub to: Address,
    pub data: Option<String>,
    pub nonce: u64,
}

impl TransferReceiptData {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

impl FutureReceiptData {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}
