import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { expect } from 'chai'
import { MarketProxy } from '../target/types/market_proxy'

describe('market_proxy', () => {
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.MarketProxy as Program<MarketProxy>

  const configKeypair = anchor.web3.Keypair.generate()
  const admin = anchor.web3.Keypair.generate()

  it('Initialize proxy with admin', async () => {
    try {
      const tx = await program.methods
        .initialize(admin.publicKey)
        .accounts({
          config: configKeypair.publicKey,
          payer: anchor.getProvider().publicKey,
        })
        .signers([configKeypair])
        .rpc()

      console.log('Initialize transaction signature', tx)

      const config = await program.account.marketConfig.fetch(configKeypair.publicKey)
      expect(config.admin.toString()).to.equal(admin.publicKey.toString())
      expect(config.currentImpl.toString()).to.equal(program.programId.toString())

    } catch (error) {
      console.log('Error:', error)
      throw error
    }
  })

  it('Update implementation', async () => {
    try {
      const newImplementation = anchor.web3.Keypair.generate().publicKey

      const tx = await program.methods
        .updateImplementation(newImplementation)
        .accounts({
          config: configKeypair.publicKey,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc()

      console.log('Update implementation transaction signature', tx)

      const config = await program.account.marketConfig.fetch(configKeypair.publicKey)
      expect(config.currentImpl.toString()).to.equal(newImplementation.toString())

    } catch (error) {
      console.log('Error:', error)
      throw error
    }
  })

  it('Fail to update implementation with wrong admin', async () => {
    try {
      const fakeAdmin = anchor.web3.Keypair.generate()
      const newImplementation = anchor.web3.Keypair.generate().publicKey

      await program.methods
        .updateImplementation(newImplementation)
        .accounts({
          config: configKeypair.publicKey,
          admin: fakeAdmin.publicKey,
        })
        .signers([fakeAdmin])
        .rpc()

      throw new Error('Should have failed with wrong admin')

    } catch (error) {
      expect(error.message).to.include('Constraint')
    }
  })

  it('[ATTACK] should reject update_implementation when the real admin pubkey is supplied but does not sign', async () => {
    try {
      const newImplementation = anchor.web3.Keypair.generate().publicKey
      await program.methods
        .updateImplementation(newImplementation)
        .accounts({
          config: configKeypair.publicKey,
          admin: admin.publicKey,
        })
        .rpc()

      throw new Error('Should have failed without the admin signature')
    } catch (error) {
      expect(error.message).to.match(/Signature verification failed|signer/i)
    }

    const config = await program.account.marketConfig.fetch(configKeypair.publicKey)
    expect(config.admin.toString()).to.equal(admin.publicKey.toString())
  })
})
