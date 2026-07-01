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

## Security Notes

This is a demo project. The following known limitations exist by design and should be addressed before any production use.

### Escrow program whitelist (not implemented)

`simple_market` accepts any `escrow_program_id` supplied by the seller at order creation time. The only validation is that the account passed as `escrow_program` matches the `escrow_program_id` stored in the order — a tautology check that does not verify the program is trustworthy.

A malicious seller could point to a custom executable that mimics the escrow interface but does not actually lock the NFT, or drains the buyer's payment without transferring the token.

**Planned fix:** `market_proxy` already has an `admin`-controlled `current_impl` field. The intended architecture is a PDA-based whitelist (`seeds = [b"allowed_escrow", program_id]`) managed by the admin via `add_escrow_program` / `remove_escrow_program` instructions. `simple_market::create_order` should verify the existence of such a PDA before accepting the `escrow_program_id`.

A non-executable (random) program address is rejected by the Solana runtime at CPI time, but an executable malicious program is not — the whitelist is the only complete defence.

### Known design trade-offs

- **No order expiry.** Orders live until explicitly cancelled or executed. There is no timeout mechanism.
- **No NFT supply validation.** The contract does not enforce `supply == 1`. A fungible token with `decimals = 0` and `supply > 1` can be listed as an NFT.
- **Seller can execute their own order.** `execute_order` does not prevent the seller from acting as the buyer. The payment flows to themselves minus transaction fees and the NFT returns to them.
- **Zero price is allowed.** `create_order` accepts `price = 0`, effectively making an order a free transfer.