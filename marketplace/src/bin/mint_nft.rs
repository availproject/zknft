
use nft_core::{
    nft::types::{NftId, NftTransactionMessage, NftTransaction, Mint, NftMetadata}, 
    types::{TxSignature, Address}
};

use serde::{ de::DeserializeOwned, Serialize, Deserialize};
use primitive_types::U256;
use reqwest::Error;
use ed25519_consensus::{Signature, SigningKey};
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
    keypair_bytes: [u8; 32]
}

#[tokio::main]
async fn main() -> Result<(), Error>  {
  let json_data = std::fs::read_to_string("keypair.json").unwrap();
  let nft_url = "http://127.0.0.1:7000/tx";
  // Deserialize the JSON data into a struct
  let keypair_data: Data = serde_json::from_str(&json_data).unwrap();

  // Create a SigningKey from the deserialized keypair_bytes
  let signing_key: SigningKey = SigningKey::from(keypair_data.keypair_bytes);
    let metadata: NftMetadata = NftMetadata {
        url: String::from("https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png"),
        description: String::from("Demo NFT, not real."), 
        name: String::from("Ape 3"),
    };
    let mut bytes = [0u8; 32];
    
    U256::from_dec_str("3").unwrap().to_big_endian(&mut bytes);
    println!("big endian: {:?}", &bytes);
    
    let mint = Mint {
        id: NftId(bytes),
        from: Address(signing_key.verification_key().to_bytes()),
        to: Address(signing_key.verification_key().to_bytes()), 
        data: None,
        future_commitment: None,
        metadata,
    };
    let nft_tx = NftTransactionMessage::Mint(mint.clone());
    let encoded_message = nft_tx.to_encoded();

    let signature: Signature = signing_key.sign(&encoded_message);

    match mint.from.verify_msg(&TxSignature::from(signature), &encoded_message)
    {
        true => { println!("Verification done")},
        false => { println!("Verification failed.")},
    };

    send_post_request(
        nft_url, 
        NftTransaction {
            message: encoded_message,
            signature: TxSignature::from(signature)
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
