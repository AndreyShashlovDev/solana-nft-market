use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};

declare_id!("BDHmaMsH8kpLB6uRwuK88LrHZogTCndF46hHBVjPCeJQ");

#[program]
pub mod fixed_price_escrow {
    use super::*;

    pub fn create(ctx: Context<CreateEscrow>, price: u64, _extra: Vec<u8>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        escrow.seller = ctx.accounts.seller.key();
        escrow.mint = ctx.accounts.mint.key();
        escrow.price = price;
        escrow.token_account = ctx.accounts.token_account.key();
        escrow.escrow_token_account = ctx.accounts.escrow_token_account.key();
        escrow.created_at = Clock::get()?.unix_timestamp as u64;
        escrow.bump = ctx.bumps.escrow;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                token::Transfer {
                    from: ctx.accounts.token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }

    pub fn execute(ctx: Context<ExecuteEscrow>, payment_amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        // check price
        require!(
            payment_amount >= escrow.price,
            EscrowError::InsufficientPayment
        );

        // Transfer SOL for seller
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.seller.key(),
            payment_amount,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
            ],
        )?;

        let escrow_bump = ctx.accounts.escrow.bump;
        let seller_key = ctx.accounts.seller.key();
        let mint_key = ctx.accounts.escrow.mint;

        let seeds = &[
            ESCROW_SEED,
            seller_key.as_ref(),
            mint_key.as_ref(),
            &[escrow_bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                token::Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[seeds],
            ),
            1, // amount (1 for NFT)
        )?;

        Ok(())
    }

    pub fn cancel(ctx: Context<CancelOrder>) -> Result<()> {
        let escrow_bump = ctx.accounts.escrow.bump;
        let seller_key = ctx.accounts.seller.key();
        let mint_key = ctx.accounts.escrow.mint;

        let seeds = &[
            ESCROW_SEED,
            seller_key.as_ref(),
            mint_key.as_ref(),
            &[escrow_bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                token::Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[seeds],
            ),
            1, // amount (1 for NFT)
        )?;

        Ok(())
    }
}

pub const ESCROW_SEED: &[u8] = b"escrow";
pub const ESCROW_TOKEN_SEED: &[u8] = b"escrow_token";

pub const DISCRIMINATOR_SIZE: usize = 8;
pub const PUBKEY_SIZE: usize = 32;
pub const U64_SIZE: usize = 8;
pub const BOOL_SIZE: usize = 1;

pub const ESCROW_SIZE: usize = DISCRIMINATOR_SIZE + // anchor
    PUBKEY_SIZE + // seller
    PUBKEY_SIZE + // mint
    U64_SIZE + // price
    PUBKEY_SIZE + // token_account,
    PUBKEY_SIZE + // escrow_token_account
    U64_SIZE + // created_at
    BOOL_SIZE; // bump

#[account]
pub struct Escrow {
    pub seller: Pubkey,
    pub mint: Pubkey,
    pub price: u64,
    pub token_account: Pubkey,
    pub escrow_token_account: Pubkey,
    pub created_at: u64,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(price: u64, extra: Vec<u8>)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = seller,
        space = ESCROW_SIZE,
        seeds = [ESCROW_SEED, seller.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // seller token account
    #[account(
        mut,
        constraint = token_account.mint == mint.key(),
        constraint = token_account.owner == seller.key(),
    )]
    pub token_account: Account<'info, TokenAccount>,

    // Используем те же сиды что и для escrow, но с другим префиксом
    #[account(
        init,
        payer = seller,
        token::mint = mint,
        token::authority = escrow, // escrow PDA будет владельцем
        seeds = [ESCROW_TOKEN_SEED, seller.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExecuteEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: from escrow account
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, seller.key().as_ref(), escrow.mint.as_ref()],
        bump = escrow.bump,
        close = seller
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = escrow_token_account.key() == escrow.escrow_token_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, seller.key().as_ref(), escrow.mint.as_ref()],
        bump = escrow.bump,
        close = seller
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = token_account.mint == escrow.mint,
        constraint = token_account.owner == seller.key()
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = escrow_token_account.key() == escrow.escrow_token_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum EscrowError {
    #[msg("Insufficient payment amount")]
    InsufficientPayment,
}
