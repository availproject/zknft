use anyhow::Error;
use ed25519_consensus::{Signature, SigningKey};
use nft_core::{
    payments::types::{CallType, Transaction, TransactionMessage},
    types::{Address, TxSignature},
    utils::hex_string_to_u8_array,
};
use primitive_types::U256;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Digest;
use sparse_merkle_tree::H256;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    keypair_bytes: [u8; 32],
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mint_to =
        String::from("0xaeb464f569c86ae89b9728c809bfae9220d0827c3c9a304262349823a180e308");
    let json_data = std::fs::read_to_string("keypair.json").unwrap();
    let payments_url = "http://127.0.0.1:7001/tx";

    // Deserialize the JSON data into a struct
    let keypair_data: Data = serde_json::from_str(&json_data).unwrap();
    // Create a SigningKey from the deserialized keypair_bytes
    let signing_key: SigningKey = SigningKey::from(keypair_data.keypair_bytes);
    let address: Address = Address(signing_key.verification_key().to_bytes());
    let to: Address = Address(hex_string_to_u8_array(&mint_to)?);

    println!("Address: {:?}", &to);
    let transaction_message: TransactionMessage = TransactionMessage {
        from: address.clone(),
        to: to.clone(),
        amount: 1000,
        call_type: CallType::Mint,
        data: None,
    };

    let encoded_message = transaction_message.to_encoded();

    let signature: Signature = signing_key.sign(&encoded_message);

    match transaction_message
        .from
        .verify_msg(&TxSignature::from(signature), &encoded_message)
    {
        true => {
            println!("Verification done")
        }
        false => {
            println!("Verification failed.")
        }
    };

    send_post_request(
        payments_url,
        Transaction {
            message: encoded_message,
            signature: TxSignature::from(signature),
        },
    )
    .await?;

    Ok(())
}

async fn send_post_request<T: Serialize + DeserializeOwned>(
    url: &str,
    body: T,
) -> Result<(), Error> {
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

async fn send_get_request<T: Serialize + DeserializeOwned, R: Serialize + DeserializeOwned>(
    base_url: &str,
    query_params: T,
) -> Result<R, Error> {
    // Create a reqwest client
    let client = reqwest::Client::new();
    // let serialized = serde_json::to_string(&body)
    // .unwrap();

    let url = format!(
        "{}?{}",
        base_url,
        serde_urlencoded::to_string(&query_params).unwrap()
    );

    // Send the POST request with the JSON body
    let response = client.get(url).send().await?;

    let parsed_response: R = response.json().await?;

    println!("GET request to {} with body completed.", base_url);

    Ok(parsed_response)
}
