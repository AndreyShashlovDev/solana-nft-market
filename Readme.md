# NFT Exchanger for the Solana Blockchain (Demo Project. As Is)

This is a demo project of an NFT exchanger for the Solana blockchain, written in Rust using the Anchor framework.

## Features

1. **Create Orders**/**Cancel Orders**/**Exchange Orders**: Sellers can create an order to sell NFTs. The token is stored in an escrow account during the transaction.
2. **Escrow Contract Implementations**: The project supports multiple implementations of escrow contracts. It includes a simple implementation with a fixed price.
3. **Proxy Contract**: There is a Proxy contract for the main market. This is useful in case the contract needs to be moved or updated, and direct updates are not possible.

## Installation - Setting Up a Local Environment for Solana Development

Follow the official Solana documentation for setting up your development environment:

[Solana Setup Guide](https://solana.com/ru/docs/intro/installation)

### Install Dependencies

To install the project dependencies, run:

```bash
yarn
```

## Running Tests and Building the Project

For instructions on how to run tests and build the project using the Anchor CLI, refer to the official documentation:

[Anchor CLI Basics - Solana Docs](https://solana.com/ru/docs/intro/installation#anchor-cli-basics)
