use nft_core::{
  nft::{
    state_machine::NftStateMachine,
    types::{Nft, NftTransaction},
  },
  app_node::{AppNode},
  types::ClientReply,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, reject::Reject, reply::Reply, Rejection};
use core::convert::Infallible;

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

pub fn nft_routes(service: Arc<Mutex<AppNode<Nft, NftTransaction, NftStateMachine>>>) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone 
{
    let listed_nfts = 
            warp::get()
            .and(warp::path("listed-nfts"))
            .and(warp::any().map(move || service.clone()))
            .and_then(get_listed_nfts);

    listed_nfts
}

