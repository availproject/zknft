use nft_core::{
  app_node::{AppNode, AppNodeConfig, RPCServer},
  payments::{
      state_machine::PaymentsStateMachine,
      types::{Account, CallType, Transaction},
  },
  traits::StateMachine,
  types::{AppChain, ClientReply},
};
use tokio::sync::Mutex;
use std::sync::Arc;
use core::convert::Infallible;
use warp::{Filter, Reply, Rejection};
use crate::types::{RPCTransaction};

pub async fn send_tx(
  service: Arc<Mutex<AppNode<Account, Transaction, PaymentsStateMachine>>>,
  call: RPCTransaction,
) ->  Result<ClientReply<String>, Infallible> {
  let transaction: Transaction = match RPCTransaction::try_into(call) {
    Ok(i) => i, 
    Err(e) => {
      println!("Bad request: {:?}", e);
      return Ok(ClientReply::BadRequest)
    },
  };
  let app = service.lock().await;
  println!("Adding transaction to pool.");

  app.add_to_tx_pool(transaction).await;

  Ok(ClientReply::Ok(String::from("Transaction added to batch.")))
}

pub fn routes(service: Arc<Mutex<AppNode<Account, Transaction, PaymentsStateMachine>>>) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone 
{
    let send_tx = warp::path("tx")
            .and(warp::any().map(move || service.clone()))
            .and(warp::body::json())
            .and_then(send_tx);

    send_tx
}
