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
