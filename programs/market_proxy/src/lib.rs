use anchor_lang::prelude::*;

declare_id!("8Hyj7sDGt9CeDYaJVn3csspBGSNA9HZktriPNBL7TDdT");

#[program]
pub mod market_proxy {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = admin;
        config.current_impl = *ctx.program_id;
        Ok(())
    }

    pub fn update_implementation(
        ctx: Context<UpdateImplementation>,
        new_implementation: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.current_impl = new_implementation;
        Ok(())
    }
}

pub const DISCRIMINATOR_SIZE: usize = 8;
pub const PUBKEY_SIZE: usize = 32;

pub const CONFIG_SIZE: usize = DISCRIMINATOR_SIZE + // anchor
    PUBKEY_SIZE + // current impl
    PUBKEY_SIZE; // admin

#[account]
pub struct MarketConfig {
    pub current_impl: Pubkey,
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = CONFIG_SIZE
    )]
    pub config: Account<'info, MarketConfig>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateImplementation<'info> {
    #[account(
        mut,
        constraint = config.admin == admin.key()
    )]
    pub config: Account<'info, MarketConfig>,

    pub admin: Signer<'info>,
}
