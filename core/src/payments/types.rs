use crate::{
    traits::{Leaf, TxHasher},
    types::{ShaHasher, TxSignature, Address},
    utils::hex_string_to_u64,
};
use risc0_zkvm::sha::rust_crypto::Digest;
use parity_scale_codec::{Encode, Decode};
use serde::{Deserialize, Serialize};
use ed25519_consensus::Signature;
use sparse_merkle_tree::{
    traits::{Hasher, Value},
    H256,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default, Encode, Decode)]
pub struct Account {
    pub address: Address,
    pub balance: u64,
    pub nonce: u64,
}

impl Leaf<H256> for Account {
    fn get_key(&self) -> H256 {
        self.address.get_key()
    }
}

impl Value for Account {
    fn to_h256(&self) -> H256 {
        if self.balance == 0 && self.nonce == 0 {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let serialized = bincode::serialize(&self).unwrap();

        hasher.0.update(&serialized);

        hasher.finish()
    }

    fn zero() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub enum CallType {
    Transfer,
    Mint,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct Transaction {
    pub message: TransactionMessage,
    pub signature: TxSignature,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct TransactionMessage {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub call_type: CallType,
    pub data: Option<String>,
}

impl TxHasher for Transaction {
    fn to_h256(&self) -> H256 {
        let mut hasher = ShaHasher::new();
        let serialized = bincode::serialize(&self).unwrap();
        hasher.0.update(&serialized);

        hasher.finish()
    }
}

impl Transaction {
    pub fn signature(&self) -> Signature {
        Signature::from(*self.signature.as_bytes())
    }
}

impl TransactionMessage {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct PaymentReceiptData {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub call_type: CallType,
    pub data: Option<String>,
    pub nonce: u64,
}

impl PaymentReceiptData {
    pub fn to_encoded(&self) -> Vec<u8> {
        self.encode()
    }
}


#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RPCTransactionMessage {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub call_type: CallType,
    pub data: Option<String>,
}


#[cfg(any(feature = "native", feature = "native-metal"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RPCTransaction {
    pub message: RPCTransactionMessage,
    pub signature: String,
}


#[cfg(any(feature = "native", feature = "native-metal"))]
impl TryFrom<RPCTransaction> for Transaction {
    type Error = anyhow::Error;

    fn try_from(rpc_transaction: RPCTransaction) -> Result<Self, Self::Error> {
        let from: Address = Address::try_from(&rpc_transaction.message.from)?;
        let to: Address = Address::try_from(&rpc_transaction.message.to)?;
        let amount: u64 = hex_string_to_u64(&rpc_transaction.message.amount)?;

        let message = TransactionMessage {
            from,
            to,
            amount,
            call_type: rpc_transaction.message.call_type,
            data: rpc_transaction.message.data,
        };

        let signature: TxSignature = TxSignature::try_from(&rpc_transaction.signature)?;

        Ok(Transaction {
            message,
            signature,
        })
    }
}
