pub mod avail;

#[cfg(not(feature = "verifier"))]
pub mod service;
// pub mod spec;

// // NOTE: Remove once dependency to the node is removed
// #[cfg(feature = "native")]
// pub use avail_subxt::build_client;
