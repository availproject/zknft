use structopt::StructOpt;
use nft_core::{
    nft::types::{NftId, Nft, Transfer, Trigger, NftTransactionMessage, Address as NftAddress, NftTransaction, Mint}, 
    payments::types::{Address, PaymentReceiptData, Transaction, CallType}, 
    types::{TransactionReceipt, TxSignature}
};
use sparse_merkle_tree::H256;
use tokio::time::Duration;
use sparse_merkle_tree::traits::Value;
use serde::{ de::DeserializeOwned, Serialize, Deserialize};
use primitive_types::U256;
use sparse_merkle_tree::MerkleProof;
use core::future::Future;
use futures::future;
use reqwest::Error;
use ed25519_dalek::SecretKey;

use rand::rngs::OsRng;
use ed25519_dalek::{Signature, SigningKey};
use sha2::Sha512;
use sha2::Digest;

struct Sell {
    pub nft_id: NftId,
    pub from: String,
    pub to: String, 
    pub amount: u64, 
    pub payment_recipient: Address, 
    pub payment_sender: Address,
    pub expected_nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    keypair_bytes: SecretKey
}

#[tokio::main]
async fn main() -> Result<(), Error>  {
  let json_data = std::fs::read_to_string("keypair.json").unwrap();
  let nft_url = "http://127.0.0.1:7000/";
  // Deserialize the JSON data into a struct
  let keypair_data: Data = serde_json::from_str(&json_data).unwrap();

  // Create a SigningKey from the deserialized keypair_bytes
  let signing_key: SigningKey = SigningKey::from_bytes(&keypair_data.keypair_bytes);
  

    let nft_tx = NftTransactionMessage::Mint(Mint {
        id: NftId(U256::from_dec_str("1").unwrap()), 
        from: NftAddress(H256::from(signing_key.verifying_key().to_bytes())),
        to: NftAddress(H256::from(signing_key.verifying_key().to_bytes())), 
        data: None,
        future_commitment: None
    });

    let mut prehashed: Sha512 = Sha512::new();

    prehashed.update(nft_tx.to_vec());

    let signature: Signature = signing_key.sign_prehashed(prehashed.clone(), None).unwrap();

    let verifying_key = signing_key.verifying_key();

    println!("Verifying key {:?}", 
    &signature
    );


    match verifying_key.verify_prehashed(prehashed, None, &signature) {
        Ok(()) => { println!("Verified."); }, 
        Err(i) => { println!("Verification failed. {:?}", i); },
    };

    send_post_request(nft_url, 
        NftTransaction {
            message: nft_tx,
            signature: TxSignature{
                r: signature.r_bytes().clone(),
                s: signature.s_bytes().clone(),
            },
        } 
    ).await?;

    Ok(())
}

async fn send_post_request<T: Serialize + DeserializeOwned>(url: &str, body: T) -> Result<(), Error> {
    // Create a reqwest client
    let client = reqwest::Client::new();

    // Send the POST request with the JSON body
    let _response = client.post(url).json(&body).send().await?;

    // Simulate some processing time
    //tokio::time::sleep(Duration::from_secs(2)).await;

    println!("POST request to {} with body completed.", url);

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct ReceiptQuery {
  key: String,
}


async fn send_get_request<T: Serialize + DeserializeOwned, R: Serialize + DeserializeOwned>(base_url: &str, query_params: T) -> Result<R, Error> {
    // Create a reqwest client
    let client = reqwest::Client::new();
    // let serialized = serde_json::to_string(&body)
    // .unwrap();

    let url = format!("{}?{}", base_url, serde_urlencoded::to_string(&query_params).unwrap());

    // Send the POST request with the JSON body
    let response = client.get(url).send().await?;

    let parsed_response: R = response.json().await?;

    println!("GET request to {} with body completed.", base_url);

    Ok(parsed_response)
}
