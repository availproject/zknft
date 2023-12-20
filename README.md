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

## Contribution Guidelines

### Rules

Avail welcomes contributors from every background and skill level. Our mission is to build a community that's not only welcoming and friendly but also aligned with the best development practices. Interested in contributing to this project? Whether you've spotted an issue, have improvement ideas, or want to add new features, we'd love to have your input. Simply open a GitHub issue or submit a pull request to get started.

1. Before asking any questions regarding how the project works, please read through all the documentation and install the project on your own local machine to try it and understand how it works. Please ask your questions in open channels (Github and [Telegram](https://t.me/avail_uncharted/5)).

2. To work on an issue, first, get approval from a maintainer or team member. You can request to be assigned by commenting on the issue in GitHub. This respects the efforts of others who may already be working on the same issue. Unapproved PRs may be declined.

3. When assigned to an issue, it's expected that you're ready to actively work on it. After assignment, please provide a draft PR or update within one week. If you encounter delays, communicate with us to maintain your assignment.

4. Got an idea or found a bug? Open an issue with the tags [New Feature] or [Bug]. Provide detailed information like reproduction steps (for bugs) or a basic feature proposal. The team will review and potentially assign you to it.

5. Start a draft PR early in your development process, even with incomplete changes. This allows us to track progress, provide timely reviews, and assist you. Expect feedback on your drafts periodically.
