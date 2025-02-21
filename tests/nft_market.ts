import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createMint,
  mintTo,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token'
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, } from '@solana/web3.js'
import { expect } from 'chai'
import { FixedPriceEscrow } from '../target/types/fixed_price_escrow'
import { SimpleMarket } from '../target/types/simple_market'

describe('NFT Market Tests', () => {
  // Common variables
  const provider = anchor.AnchorProvider.env()
  const marketProgram = anchor.workspace.SimpleMarket as Program<SimpleMarket>
  const escrowProgram = anchor.workspace.FixedPriceEscrow as Program<FixedPriceEscrow>

  // Test accounts and constants
  const seller = Keypair.generate()
  const buyer = Keypair.generate()
  const PRICE = new anchor.BN(1 * LAMPORTS_PER_SOL)
  const EXTRA_DATA = Buffer.from(JSON.stringify({name: 'Test NFT'}))
  const AIRDROP_AMOUNT = 2 * LAMPORTS_PER_SOL

  // Test state variables
  let nftMint: PublicKey
  let sellerTokenAccount: PublicKey
  let buyerTokenAccount: PublicKey
  let orderPda: PublicKey
  let mintToOrderPda: PublicKey
  let escrowPda: PublicKey
  let escrowTokenAccount: PublicKey

  async function setupNFT() {
    nftMint = await createMint(
      provider.connection,
      seller,
      seller.publicKey,
      null,
      0
    )

    sellerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      seller,
      nftMint,
      seller.publicKey
    )

    buyerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      buyer,
      nftMint,
      buyer.publicKey
    )

    await mintTo(
      provider.connection,
      seller,
      nftMint,
      sellerTokenAccount,
      seller.publicKey,
      1
    )
  }

  async function derivePDAs() {
    [orderPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('order'),
        seller.publicKey.toBuffer(),
        nftMint.toBuffer()
      ],
      marketProgram.programId
    );

    [mintToOrderPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('mint_to_order'),
        nftMint.toBuffer()
      ],
      marketProgram.programId
    );

    [escrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('escrow'),
        seller.publicKey.toBuffer(),
        nftMint.toBuffer()
      ],
      escrowProgram.programId
    );

    [escrowTokenAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('escrow_token'),
        seller.publicKey.toBuffer(),
        nftMint.toBuffer(),
      ],
      escrowProgram.programId
    )
  }

  beforeEach(async () => {
    anchor.setProvider(provider)

    try {
      // Airdrop SOL to seller and buyer
      const signatures = await Promise.all([
        provider.connection.requestAirdrop(seller.publicKey, AIRDROP_AMOUNT),
        provider.connection.requestAirdrop(buyer.publicKey, AIRDROP_AMOUNT)
      ])

      await Promise.all(
        signatures.map(sig => provider.connection.confirmTransaction(sig))
      )

      // Setup NFT and derive PDAs
      await setupNFT()
      await derivePDAs()
    } catch (error) {
      console.error('Error in test setup:', error)
      throw error
    }
  })

  it('should create an order', async () => {
    const sellerTokenBalanceBefore = Number(
      (await provider.connection.getTokenAccountBalance(sellerTokenAccount)).value.amount
    )

    await marketProgram.methods
      .createOrder(PRICE, escrowProgram.programId, EXTRA_DATA)
      .accounts({
        seller: seller.publicKey,
        nftMint: nftMint,
        order: orderPda,
        mintToOrder: mintToOrderPda,
        escrowProgram: escrowProgram.programId,
        escrowAccount: escrowPda,
        escrowTokenAccount: escrowTokenAccount,
        tokenAccount: sellerTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([seller])
      .rpc()

    // Verify order account
    const order = await marketProgram.account.order.fetch(orderPda)
    expect(order.seller.toString()).to.equal(seller.publicKey.toString())
    expect(order.escrowProgram.toString()).to.equal(escrowProgram.programId.toString())
    expect(order.escrowAccount.toString()).to.equal(escrowPda.toString())
    expect(order.order.price.toString()).to.equal(PRICE.toString())
    expect(order.order.mint.toString()).to.equal(nftMint.toString())

    // Verify mint to order account
    const mintToOrder = await marketProgram.account.mintToOrder.fetch(mintToOrderPda)
    expect(mintToOrder.mint.toString()).to.equal(nftMint.toString())
    expect(mintToOrder.order.toString()).to.equal(orderPda.toString())

    // Verify escrow account
    const escrow = await escrowProgram.account.escrow.fetch(escrowPda)
    expect(escrow.seller.toString()).to.equal(seller.publicKey.toString())
    expect(escrow.mint.toString()).to.equal(nftMint.toString())
    expect(escrow.price.toString()).to.equal(PRICE.toString())
    expect(escrow.tokenAccount.toString()).to.equal(sellerTokenAccount.toString())

    // Verify token balance
    const sellerTokenBalanceAfter = Number(
      (await provider.connection.getTokenAccountBalance(sellerTokenAccount)).value.amount
    )
    expect(sellerTokenBalanceBefore).to.equal(1)
    expect(sellerTokenBalanceAfter).to.equal(0)
  })

  it('should cancel an order', async () => {
    // Create order first
    await marketProgram.methods
      .createOrder(PRICE, escrowProgram.programId, EXTRA_DATA)
      .accounts({
        seller: seller.publicKey,
        nftMint: nftMint,
        order: orderPda,
        mintToOrder: mintToOrderPda,
        escrowProgram: escrowProgram.programId,
        escrowAccount: escrowPda,
        escrowTokenAccount: escrowTokenAccount,
        tokenAccount: sellerTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([seller])
      .rpc()

    const sellerTokenBalanceBefore = Number(
      (await provider.connection.getTokenAccountBalance(sellerTokenAccount)).value.amount
    )
    const sellerBalanceBefore = await provider.connection.getBalance(seller.publicKey)

    // Cancel order
    await marketProgram.methods
      .cancelOrder()
      .accounts({
        seller: seller.publicKey,
        order: orderPda,
        escrowAccount: escrowPda,
        mintToOrder: mintToOrderPda,
        escrowTokenAccount,
        tokenAccount: sellerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        escrowProgram: escrowProgram.programId,
      })
      .signers([seller])
      .rpc()

    // Verify accounts are closed
    const [orderAccount, mintToOrderAccount, escrowAccount] = await Promise.all([
      provider.connection.getAccountInfo(orderPda),
      provider.connection.getAccountInfo(mintToOrderPda),
      provider.connection.getAccountInfo(escrowPda)
    ])

    expect(orderAccount).to.be.null
    expect(mintToOrderAccount).to.be.null
    expect(escrowAccount).to.be.null

    // Verify balances
    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey)
    const sellerTokenBalanceAfter = Number(
      (await provider.connection.getTokenAccountBalance(sellerTokenAccount)).value.amount
    )

    expect(sellerBalanceAfter).to.be.greaterThan(sellerBalanceBefore)
    expect(sellerTokenBalanceBefore).to.equal(0)
    expect(sellerTokenBalanceAfter).to.equal(1)
  })

  it('should execute an order', async () => {
    // Create order first
    await marketProgram.methods
      .createOrder(PRICE, escrowProgram.programId, EXTRA_DATA)
      .accounts({
        seller: seller.publicKey,
        nftMint: nftMint,
        order: orderPda,
        mintToOrder: mintToOrderPda,
        escrowProgram: escrowProgram.programId,
        escrowAccount: escrowPda,
        escrowTokenAccount: escrowTokenAccount,
        tokenAccount: sellerTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([seller])
      .rpc()

    // Get initial balances
    const [
      sellerBalanceBefore,
      buyerBalanceBefore,
      sellerTokenBalanceBefore,
      buyerTokenBalanceBefore
    ] = await Promise.all([
      provider.connection.getBalance(seller.publicKey),
      provider.connection.getBalance(buyer.publicKey),
      provider.connection.getTokenAccountBalance(sellerTokenAccount),
      provider.connection.getTokenAccountBalance(buyerTokenAccount)
    ])

    // Execute order
    await marketProgram.methods
      .executeOrder(PRICE)
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        order: orderPda,
        mintToOrder: mintToOrderPda,
        escrowAccount: escrowPda,
        escrowTokenAccount,
        buyerTokenAccount,
        mint: nftMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        escrowProgram: escrowProgram.programId,
      })
      .signers([buyer])
      .rpc()

    // Verify accounts are closed
    const [orderAccount, mintToOrderAccount, escrowAccount] = await Promise.all([
      provider.connection.getAccountInfo(orderPda),
      provider.connection.getAccountInfo(mintToOrderPda),
      provider.connection.getAccountInfo(escrowPda)
    ])

    expect(orderAccount).to.be.null
    expect(mintToOrderAccount).to.be.null
    expect(escrowAccount).to.be.null

    // Get final balances
    const [
      sellerBalanceAfter,
      buyerBalanceAfter,
      sellerTokenBalanceAfter,
      buyerTokenBalanceAfter
    ] = await Promise.all([
      provider.connection.getBalance(seller.publicKey),
      provider.connection.getBalance(buyer.publicKey),
      provider.connection.getTokenAccountBalance(sellerTokenAccount),
      provider.connection.getTokenAccountBalance(buyerTokenAccount)
    ])

    // Verify payment transfer
    expect(sellerBalanceAfter - sellerBalanceBefore).to.be.approximately(
      PRICE.toNumber(),
      0.1 * LAMPORTS_PER_SOL // Account for transaction fees
    )

    expect(buyerBalanceBefore - buyerBalanceAfter).to.be.approximately(
      PRICE.toNumber(),
      0.1 * LAMPORTS_PER_SOL // Account for transaction fees
    )

    // Verify NFT transfer
    expect(Number(sellerTokenBalanceBefore.value.amount)).to.equal(0)
    expect(Number(sellerTokenBalanceAfter.value.amount)).to.equal(0)
    expect(Number(buyerTokenBalanceBefore.value.amount)).to.equal(0)
    expect(Number(buyerTokenBalanceAfter.value.amount)).to.equal(1)

    // Security checks
    try {
      await marketProgram.methods
        .executeOrder(PRICE)
        .accounts({
          buyer: buyer.publicKey,
          seller: seller.publicKey,
          order: orderPda,
          mintToOrder: mintToOrderPda,
          escrowAccount: escrowPda,
          escrowTokenAccount,
          buyerTokenAccount,
          mint: nftMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          escrowProgram: escrowProgram.programId,
        })
        .signers([buyer])
        .rpc()
      expect.fail('Should not be able to execute order twice')
    } catch (error) {
      // Expected error
    }

    try {
      await marketProgram.methods
        .cancelOrder()
        .accounts({
          seller: seller.publicKey,
          order: orderPda,
          escrowAccount: escrowPda,
          mintToOrder: mintToOrderPda,
          escrowTokenAccount,
          tokenAccount: sellerTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          escrowProgram: escrowProgram.programId,
        })
        .signers([seller])
        .rpc()
      expect.fail('Should not be able to cancel executed order')
    } catch (error) {
      // Expected error
    }
  })
})
