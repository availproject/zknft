# ZKNFT

Welcome to an experimental project that delves into the realm of asynchronous composability in blockchains. This initiative aims to facilitate seamless communication and information sharing among diverse application chains. The following example illustrates a scenario where an NFT purchase occurs on one chain while the payment is accepted on a different chain, showcasing the exciting possibilities unlocked.

## Nexus

The Nexus crate serves as a middleware that plays a crucial role in the project's architecture. It performs the following key functions:

- Accepts batches from application chains and verifies zk-proofs.
- Generates an aggregated succinct proof for all the accepted batches.
- Aggregates all receipts generated in each app chain, to create a receipt merkle root.

The receipt root enables the proof of inclusion for a receipt on any app chain to be validated on any other app chain. This functionality ensures a high level of interoperability among different chains, and offers increased flexibility.

## Core

The Core package provides essential tooling for creating application chains. These chains are designed to seamlessly communicate with the Nexus middleware, forming a cohesive and interconnected blockchain ecosystem.

## NFTApp

The NFTApp crate represents an application chain specifically tailored for Non-Fungible Tokens (NFTs). This chain interacts with the Nexus middleware to leverage the benefits of asynchronous composability.

## PaymentsApp

The PaymentsApp crate is dedicated to handling payment-related functionalities within the blockchain ecosystem. It communicates with the Nexus middleware to ensure that payment information is securely and efficiently shared across different application chains.

## NFT UI

The NFT UI serves as the user interface for interacting with the NFT application chain. It provides users with a seamless experience for managing and trading NFTs within the blockchain network.

## Payments UI

The Payments UI is the user interface designed for interacting with the Payments application chain. It enables users to engage with payment functionalities and ensures a smooth user experience in the blockchain ecosystem.

## Getting Started

To get started with this project, follow these steps:

1. Clone the repository: `git clone https://github.com/availproject/zknft.git`
2. Navigate to the project directory: `cd zknft`
3. Follow the specific README files in each crate to set up and run the respective components.
