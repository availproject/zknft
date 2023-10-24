use nft_core::{
  payments::{
      types::{TransactionMessage, Transaction},
  },
  types::TxSignature,
};
use serde::{Deserialize, Serialize};
use parity_scale_codec::Decode;
use serde_big_array::BigArray;
use std::convert::TryInto;
use anyhow::anyhow;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RPCTransaction {
  message: Vec<u8>, 
  #[serde(with = "BigArray")]
  signature: [u8; 64],
}

impl TryInto<Transaction> for RPCTransaction {
  type Error = anyhow::Error;

  fn try_into(self) -> Result<Transaction, Self::Error> {
    let mut vec_u8 = self.message.clone();
    let mut slice_u8: &[u8] = &vec_u8;

    let message: TransactionMessage = match TransactionMessage::decode(&mut slice_u8) {
      Ok(i) => i, 
      Err(e) => return Err(anyhow!("{:?}", e)),
    };

    Ok(Transaction {
      message, 
      signature: TxSignature::from(self.signature.clone())
    })
  }
}
