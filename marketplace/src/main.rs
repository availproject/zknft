use structopt::StructOpt;
use nft_core::{
    nft::types::{NftId, Nft, Transfer, Trigger, NftTransaction}, 
    payments::types::{Address, PaymentReceiptData, Transaction, CallType}, 
    types::TransactionReceipt
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

struct Sell {
    pub nft_id: NftId,
    pub from: String,
    pub to: String, 
    pub amount: u64, 
    pub payment_recipient: Address, 
    pub payment_sender: Address,
    pub expected_nonce: u64,
}

#[tokio::main]
async fn main() -> Result<(), Error>  {
    let rpc_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            loop {
                println!("");
            }
        })
    });
    // let nexus_url = "http://127.0.0.1:8080/receipt";
    // let payments_url = "http://127.0.0.1:7001/";
    // let nft_url = "http://127.0.0.1:7000/";

    // let sell_command = Sell {
    //     payment_sender: Address(H256::from( [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1])), 
    //     payment_recipient: Address(H256::from( [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2])),
    //     expected_nonce: 2,
    //     amount: 10, 
    //     nft_id: NftId(U256::from_dec_str("1").unwrap()), 
    //     from: String::from("ABCD"), 
    //     to: String::from("EFGH"),
    // };


    // let expected_receipt_data = PaymentReceiptData {
    //     from: sell_command.payment_sender.clone(), 
    //     to: sell_command.payment_recipient.clone(),
    //     amount: sell_command.amount, 
    //     call_type: CallType::Transfer, 
    //     nonce: sell_command.expected_nonce,
    //     data: None,
    // };
    // let transaction_receipt = TransactionReceipt {
    //     chain_id: 7001, 
    //     data: expected_receipt_data.to_vec()
    // };
    // let commitment_hash = transaction_receipt.to_h256();

    // let nft_tx = Transfer {
    //     id: sell_command.nft_id.clone(), 
    //     from: sell_command.from.clone(), 
    //     to: sell_command.to.clone(), 
    //     data: None, 
    //     future_commitment: Some(commitment_hash.clone())
    // };
    // //let serialized_nft_tx = serde_json::to_string(&nft_tx)?;

    // let payment_tx = Transaction {
    //     from: sell_command.payment_sender.clone(), 
    //     to: sell_command.payment_recipient.clone(), 
    //     amount: sell_command.amount, 
    //     call_type: CallType::Transfer, 
    //     data: None,
    // };
    // // let serialized_payment_tx = serde_json::to_string(&payment_tx)?;

    // let (result1, result2) = future::join(
    //     send_post_request(nft_url, NftTransaction::Transfer(nft_tx.clone()))
    //     ,
    //     send_post_request(payments_url, payment_tx.clone())
    // ).await;

    // match (result1, result2) {
    //     (Ok(()), Ok(())) => {
    //         println!("Both POST requests completed successfully.");
    //         ()
    //     }
    //     (Err(e1), Err(e2)) => {
    //         return Err(e1);
    //     }
    //     (Err(e), _) | (_, Err(e)) => {
    //         return Err(e);
    //     }
    // }

    // //Wait 15 seconds for proof to be aggregated.
    // tokio::time::sleep(Duration::from_secs(10)).await;
    // let query = ReceiptQuery {
    //     key: hex::encode(commitment_hash.as_slice())
    // };

    // let aggregated_proof: (TransactionReceipt, MerkleProof) = send_get_request(nexus_url, query).await?;

    // let trigger_tx = Trigger {
    //     id: sell_command.nft_id, 
    //     from: sell_command.from, 
    //     data: None, 
    //     merkle_proof: aggregated_proof.1, 
    //     receipt: aggregated_proof.0,
    // };

    // send_post_request(nft_url, NftTransaction::Trigger(trigger_tx.clone())).await?;

    // Ok(())
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
