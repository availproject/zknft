use nft_core::{
  nft::{
    state_machine::NftStateMachine,
    types::{Nft, NftTransaction},
  },
  app_node::{AppNode}
};
use std::sync::Arc;
use tokio::sync::Mutex;
use actix_web::{web, HttpResponse, Responder};

pub async fn get_listed_nfts(
  service: web::Data<Arc<Mutex<AppNode<Nft, NftTransaction, NftStateMachine>>>>, 
) -> impl Responder {
  let app = service.lock().await;
  let state_machine = app.state_machine.lock().await;

  match state_machine.get_listed_nfts() {
    Ok(result) => {
        // Serialize the result to JSON
        let json_result = serde_json::to_string(&result);

        match json_result {
            Ok(json_str) => {
                // Return a JSON response with a status code
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json_str)
            }
            Err(_) => HttpResponse::InternalServerError().finish(), // Handle serialization error
        }
    }
    Err(_) => HttpResponse::InternalServerError().finish(), // Handle an error from get_listed_nfts
  }
}