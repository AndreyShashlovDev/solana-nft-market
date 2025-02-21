use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke, pubkey::Pubkey};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

declare_id!("D4R5iopiYTznXp2yKq7zTZHAGyVq77Qp9N15cCBnWaYn");


#[program]
pub mod simple_market {
    use super::*;

    pub fn create_order(
        ctx: Context<CreateOrder>,
        price: u64,
        escrow_program_id: Pubkey,
        extra: Vec<u8>,
    ) -> Result<()> {
        let clock = Clock::get()?;

        // Create order
        let order = &mut ctx.accounts.order;
        order.seller = ctx.accounts.seller.key();
        order.escrow_program = escrow_program_id;
        order.escrow_account = ctx.accounts.escrow_account.key();
        order.order = OrderData {
            price,
            mint: ctx.accounts.nft_mint.key(),
            created_at: clock.unix_timestamp as u64,
        };
        order.bump = ctx.bumps.order;
        order.extra = extra.clone();

        let mint_to_order = &mut ctx.accounts.mint_to_order;
        mint_to_order.mint = ctx.accounts.nft_mint.key();
        mint_to_order.order = order.key();
        mint_to_order.bump = ctx.bumps.mint_to_order;

        let mut data = vec![];
        // create discriminator
        data.extend_from_slice(&[24, 30, 200, 40, 5, 28, 7, 119]);
        data.extend_from_slice(&price.to_le_bytes());
        data.extend_from_slice(&(extra.len() as u32).to_le_bytes());
        data.extend_from_slice(&extra);

        let create_escrow_ix = Instruction {
            program_id: escrow_program_id,
            accounts: vec![
                AccountMeta::new(ctx.accounts.seller.key(), true),
                AccountMeta::new_readonly(ctx.accounts.nft_mint.key(), false),
                AccountMeta::new(ctx.accounts.escrow_account.key(), false),
                AccountMeta::new(ctx.accounts.token_account.key(), false),
                AccountMeta::new(ctx.accounts.escrow_token_account.key(), false),
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.rent.key(), false),
            ],
            data,
        };

        invoke(
            &create_escrow_ix,
            &[
                ctx.accounts.seller.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.token_account.to_account_info(),
                ctx.accounts.escrow_token_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn execute_order(ctx: Context<ExecuteOrder>, payment_amount: u64) -> Result<()> {
        let order = &ctx.accounts.order;

        let mut data = vec![];
        // execute discriminator
        data.extend_from_slice(&[130, 221, 242, 154, 13, 193, 189, 29]);
        data.extend_from_slice(&payment_amount.to_le_bytes());

        let execute_ix = Instruction {
            program_id: order.escrow_program,
            accounts: vec![
                AccountMeta::new(ctx.accounts.buyer.key(), true),
                AccountMeta::new(ctx.accounts.seller.key(), false),
                AccountMeta::new(ctx.accounts.escrow_account.key(), false),
                AccountMeta::new(ctx.accounts.escrow_token_account.key(), false),
                AccountMeta::new(ctx.accounts.buyer_token_account.key(), false),
                AccountMeta::new_readonly(ctx.accounts.associated_token_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.mint.key(), false),
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.rent.key(), false),
            ],
            data,
        };

        invoke(
            &execute_ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.escrow_token_account.to_account_info(),
                ctx.accounts.buyer_token_account.to_account_info(),
                ctx.accounts.associated_token_program.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
        let mut data = vec![];
        // cancel discriminator
        data.extend_from_slice(&[232, 219, 223, 41, 219, 236, 220, 190]);

        let cancel_ix = Instruction {
            program_id: ctx.accounts.order.escrow_program,
            accounts: vec![
                AccountMeta::new(ctx.accounts.seller.key(), true),
                AccountMeta::new(ctx.accounts.escrow_account.key(), false),
                AccountMeta::new(ctx.accounts.token_account.key(), false),
                AccountMeta::new(ctx.accounts.escrow_token_account.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            ],
            data,
        };

        invoke(
            &cancel_ix,
            &[
                ctx.accounts.seller.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.token_account.to_account_info(),
                ctx.accounts.escrow_token_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}


pub const ORDER_SEED: &[u8] = b"order";
pub const MINT_TO_ORDER_SEED: &[u8] = b"mint_to_order";

pub const DISCRIMINATOR_SIZE: usize = 8;
pub const PUBKEY_SIZE: usize = 32;
pub const U64_SIZE: usize = 8;
pub const BOOL_SIZE: usize = 1;
pub const VEC_PREFIX_SIZE: usize = 4;

pub const ORDER_SIZE: usize = DISCRIMINATOR_SIZE + // anchor discriminator
    PUBKEY_SIZE + // seller
    PUBKEY_SIZE + // escrow_program
    PUBKEY_SIZE + // escrow_account
    PUBKEY_SIZE + // order.mint
    U64_SIZE + // order.price
    U64_SIZE + // order.created_at
    VEC_PREFIX_SIZE + // extra vec length
    512 + // extra data max size
    BOOL_SIZE; // bump

pub const MINT_TO_ORDER_SIZE: usize = DISCRIMINATOR_SIZE + // anchor
    PUBKEY_SIZE + // mint
    PUBKEY_SIZE + // order
    BOOL_SIZE; // bump

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OrderData {
    pub price: u64,
    pub mint: Pubkey,
    pub created_at: u64,
}

#[account]
pub struct Order {
    pub seller: Pubkey,
    pub escrow_program: Pubkey,
    pub escrow_account: Pubkey,
    pub order: OrderData,
    pub extra: Vec<u8>, // some additional json data for some escrow implementations
    pub bump: u8,
}

#[account]
pub struct MintToOrder {
    pub mint: Pubkey,
    pub order: Pubkey,
    pub bump: u8,
}


#[derive(Accounts)]
#[instruction(price: u64, escrow_program_id: Pubkey)]
pub struct CreateOrder<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = seller,
        space = ORDER_SIZE,
        seeds = [ORDER_SEED, seller.key().as_ref(), nft_mint.key().as_ref()],
        bump
    )]
    pub order: Account<'info, Order>,

    #[account(
        init,
        payer = seller,
        space = MINT_TO_ORDER_SIZE,
        seeds = [MINT_TO_ORDER_SEED, nft_mint.key().as_ref()],
        bump
    )]
    pub mint_to_order: Account<'info, MintToOrder>,

    /// CHECK: Validated via constraint
    #[account(constraint = escrow_program.key() == escrow_program_id @ MarketError::InvalidEscrowProgram
    )]
    pub escrow_program: UncheckedAccount<'info>,

    /// CHECK: Created via CPI to escrow program
    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    /// CHECK: Created via CPI to escrow program
    #[account(mut)]
    pub escrow_token_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = token_account.mint == nft_mint.key(),
        constraint = token_account.owner == seller.key()
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(payment_amount: u64)]
pub struct ExecuteOrder<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Validated in escrow program
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ORDER_SEED, seller.key().as_ref(), order.order.mint.as_ref()],
        bump = order.bump,
        constraint = order.seller == seller.key(),
        close = seller
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [MINT_TO_ORDER_SEED, order.order.mint.as_ref()],
        bump = mint_to_order.bump,
        constraint = mint_to_order.order == order.key(),
        close = seller
    )]
    pub mint_to_order: Account<'info, MintToOrder>,

    /// CHECK: Validated via constraint
    #[account(mut, constraint = escrow_account.key() == order.escrow_account)]
    pub escrow_account: AccountInfo<'info>,

    /// CHECK: escrow token account
    #[account(mut)]
    pub escrow_token_account: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// CHECK: Validated via constraint
    #[account(constraint = escrow_program.key() == order.escrow_program @ MarketError::InvalidEscrowProgram
    )]
    pub escrow_program: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [ORDER_SEED, seller.key().as_ref(), order.order.mint.as_ref()],
        bump = order.bump,
        constraint = order.seller == seller.key(),
        close = seller
    )]
    pub order: Account<'info, Order>,

    /// CHECK: Validated via constraint
    #[account(
        mut,
        constraint = escrow_account.key() == order.escrow_account,
    )]
    pub escrow_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [MINT_TO_ORDER_SEED, order.order.mint.as_ref()],
        bump = mint_to_order.bump,
        constraint = mint_to_order.order == order.key(),
        close = seller
    )]
    pub mint_to_order: Account<'info, MintToOrder>,

    #[account(
        mut,
        constraint = token_account.mint == order.order.mint,
        constraint = token_account.owner == seller.key()
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: Validated via constraint
    #[account(constraint = escrow_program.key() == order.escrow_program @ MarketError::InvalidEscrowProgram
    )]
    pub escrow_program: UncheckedAccount<'info>,

    /// CHECK: escrow token account
    #[account(mut)]
    pub escrow_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum MarketError {
    #[msg("Invalid escrow program address")]
    InvalidEscrowProgram,
}
