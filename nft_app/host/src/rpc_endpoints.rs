use nft_core::{
  nft::{
    state_machine::NftStateMachine,
    types::{Nft, NftTransaction, Transfer, NftTransactionMessage, NftId, Future, Trigger},
  },
  app_node::{AppNode},
  payments::types::{PaymentReceiptData, CallType, Account},
  types::{ClientReply, TxSignature, TransactionReceipt, Address, ShaHasher},
  utils::{hex_string_to_u8_array, u8_array_to_hex_string},
};
use sparse_merkle_tree::MerkleProof;
use sparse_merkle_tree::{traits::Value, H256};
use ed25519_consensus::{SigningKey, Signature};
use primitive_types::U256;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, reply::Reply, Rejection};
use core::convert::Infallible;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Error};
use crate::types::ReceiptQuery;

const NFT_PRICE:u64 = 10;
const NEXUS_RECEIPT_URL: &str = "http://127.0.0.1:8080/receipt";

pub async fn get_listed_nfts(
  service: Arc<Mutex<AppNode<Nft, NftTransaction, NftStateMachine>>>, 
) -> Result<ClientReply<Vec<Nft>>, Infallible> {
  println!("Getting NFTs");
  let app = service.lock().await;
  let state_machine = app.state_machine.lock().await;

  match state_machine.get_listed_nfts() {
    Ok(result) => Ok(ClientReply::Ok(result)),
    Err(e) => Ok(ClientReply::Error(e)),
  }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BuyNftQuery {
  nft_id: String, 
  payment_sender: String,
  nft_receiver: String,
}

pub async fn check_payment(
  key: SigningKey, 
  service: Arc<Mutex<AppNode<Nft, NftTransaction, NftStateMachine>>>,
  id: String, 
) -> Result<ClientReply<TransactionReceipt>, Infallible> {
  let verifying_key = Address(key.verification_key().to_bytes());
  let mut bytes = [0u8; 32];
  U256::from_dec_str(&id).unwrap().to_big_endian(&mut bytes);
  let nft_id = NftId(bytes);
  let app = service.lock().await;

  let nft_future: Future = match app.get_state_with_proof(&H256::from(nft_id.0)).await {
    Ok((nft, merkle_proof)) => {
      let root = match app.get_root().await {
        Ok(i) => i, 
        Err(e) => return Ok(ClientReply::Error(e))
      };

      let result = merkle_proof.verify::<ShaHasher>(&root, vec![(nft_id.get_key(), nft.to_h256())]);

      println!("rpc merkle: {:?}", &result);

      match nft.future {
        None => return Ok(ClientReply::Error(anyhow!("NFT transfer not initiated."))), 
        Some(i) => i,
      }
    },
    Err(e) => return Ok(ClientReply::Error(e))
  };

  let receipt_query: ReceiptQuery = ReceiptQuery {
    key: u8_array_to_hex_string(nft_future.commitment.as_slice())
  };

  println!("receipt queryy: {:?}", &receipt_query);

  let url = match reqwest::Url::parse_with_params(NEXUS_RECEIPT_URL, &[("key", &receipt_query.key)]) {
    Ok(i) => i, 
    Err(e) => return Ok(ClientReply::Error(e.into()))
  };

  // Perform the GET request using the constructed URL
  let response = match reqwest::get(url.as_str()).await {
    Ok(i) => i, 
    Err(e) => return Ok(ClientReply::Error(e.into()))
  };
  let (receipt, proof): (TransactionReceipt, MerkleProof) = match response.json().await {
    Ok(i) => i, 
    Err(e) => return Ok(ClientReply::Error(e.into()))
  };

  if receipt == TransactionReceipt::zero() {
    return Ok(ClientReply::Error(anyhow!("Receipt not aggregated yet.")))
  }

  let trigger = Trigger {
    id: nft_id, 
    from: verifying_key, 
    data: None,
    merkle_proof: proof, 
    receipt: receipt.clone(),
  };

  let tx_message: NftTransactionMessage = NftTransactionMessage::Trigger(trigger);
  let encoded_message = tx_message.to_encoded();
  let signature: Signature = key.sign(&encoded_message);
  let tx = NftTransaction {
    message: encoded_message,
    signature: TxSignature::from(signature)
  };
  
  println!("Adding to pool");
  app.add_to_tx_pool(tx).await;

  Ok(ClientReply::Ok(receipt))
}

async fn get_nonce(key: &str) -> Result<u64, Error> {
  //TODO: Make this url configurable.
  let url = format!("http://localhost:7001/state/{}", &key);

  println!("Sending to url: {:?}", &url);
  // Create a Reqwest client
  let client = reqwest::Client::new();

  // Send a GET request to the Warp endpoint
  let response = client.get(url).send().await?;

  // Check if the request was successful
  if response.status().is_success() {
      // Read the response as a string
      let response_body = response.text().await?;
      println!("Response: {}", response_body);

    // Deserialize the response as JSON
    let parsed_response: (Account, MerkleProof) = serde_json::from_str(&response_body)?;

   return Ok(parsed_response.0.nonce);
  } else {
      return Err(anyhow!("Could not get nonce."))
  }
}

pub async fn buy_listed_nft(
  key_service: (SigningKey, Arc<Mutex<AppNode<Nft, NftTransaction, NftStateMachine>>>),
  params: BuyNftQuery,
) -> Result<ClientReply<String>, Infallible> {
  let service = key_service.1;
  let signing_key = key_service.0;
  let verifying_key = Address(signing_key.verification_key().to_bytes());
  let nft_to = Address(match hex_string_to_u8_array(&params.nft_receiver){
    Ok(i) => i, 
    Err(e) => return Ok(ClientReply::Error(e)),
  });
  let payments_sender = Address(match hex_string_to_u8_array(&params.payment_sender) {
    Ok(i) => i, 
    Err(e) => return Ok(ClientReply::Error(e)),
  });
  let mut bytes = [0u8; 32];
  let nonce = match  get_nonce(&params.payment_sender).await {
    Ok(i) => i + 1,
    Err(e) => return Ok(ClientReply::Error(e)),
  };
    
  U256::from_dec_str(&params.nft_id).unwrap().to_big_endian(&mut bytes);
  let nft_id = NftId(bytes);

  let expected_receipt_data = PaymentReceiptData {
        from: payments_sender, 
        to: verifying_key.clone(),
        amount: NFT_PRICE, 
        call_type: CallType::Transfer, 
        nonce,
        data: None,
  };

  let transaction_receipt = TransactionReceipt {
    chain_id: 7001, 
    data: expected_receipt_data.to_encoded()
  };

  let commitment_hash = transaction_receipt.to_h256();

  let transfer = Transfer {
    id: nft_id.clone(), 
    from: verifying_key.clone(), 
    to: nft_to, 
    data: None, 
    future_commitment: Some(commitment_hash.clone())
  };

  let nft_tx = NftTransactionMessage::Transfer(transfer.clone());
  let encoded_message = nft_tx.to_encoded();

  let signature: Signature = signing_key.sign(&encoded_message);

  let app = service.lock().await;
  let call = NftTransaction {
    message: encoded_message,
    signature: TxSignature::from(signature)
  };

  match app.get_state_with_proof(&H256::from(nft_id.0)).await {
    Ok((nft, i)) => {
      if nft.owner == verifying_key {
        ()
      } else {
        return Ok(ClientReply::Error(anyhow!("NFT not listed.")))
      }
    },
    Err(e) => return Ok(ClientReply::Error(e))
  };

  println!("Adding this to pool: {:?}", &call);

  app.add_to_tx_pool(call).await;

  Ok(ClientReply::Ok(String::from("Transaction added to batch.")))
}

pub fn nft_routes(
  service: Arc<Mutex<AppNode<Nft, NftTransaction, NftStateMachine>>>,
  signing_key: SigningKey,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone 
{
    let buy_nft_app = service.clone();
    let listed_nfts_app = service.clone();
    let check_payment_app = service.clone();
    let check_payment_signing_key = signing_key.clone();

    let listed_nfts = warp::get()
    .and(warp::path("listed-nfts"))
    .and(warp::any().map(move || listed_nfts_app.clone()))
    .and_then(get_listed_nfts);

    let buy_nft = warp::post()
    .and(warp::path("buy-nft"))
    .and(warp::any().map(move || (signing_key.clone(), buy_nft_app.clone())))
    .and(warp::body::json())
    .and_then(buy_listed_nft);

    let check_payment = warp::get()
    .and(warp::path("check-payment"))
    .and(warp::any().map(move || check_payment_signing_key.clone()))
    .and(warp::any().map(move || check_payment_app.clone()))
    .and(warp::path::param::<String>())
    .and_then(check_payment);

    listed_nfts.or(buy_nft).or(check_payment)
}

