use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::Pool;
use crate::events::PoolInitialized;
use crate::errors::AmmError;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = Pool::LEN,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = payer,
        seeds = [b"vault_a", pool.key().as_ref()],
        bump,
        token::mint = token_a_mint,
        token::authority = pool,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [b"vault_b", pool.key().as_ref()],
        bump,
        token::mint = token_b_mint,
        token::authority = pool,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [b"lp_mint", pool.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub lp_mint: Account<'info, Mint>,

    /// CHECK: This is the account that will receive protocol fees. Can be any account.
    pub protocol_fee_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    fee_rate_bps: u16,
    protocol_fee_rate_bps: u16,
) -> Result<()> {
    require!(fee_rate_bps <= 10000, AmmError::InvalidFeeRate); // Max 100%
    require!(protocol_fee_rate_bps <= 10000, AmmError::InvalidFeeRate);

    let pool = &mut ctx.accounts.pool;
    pool.token_a_mint = ctx.accounts.token_a_mint.key();
    pool.token_b_mint = ctx.accounts.token_b_mint.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.fee_rate_bps = fee_rate_bps;
    pool.protocol_fee_rate_bps = protocol_fee_rate_bps;
    pool.protocol_fee_vault = ctx.accounts.protocol_fee_vault.key();
    pool.total_liquidity = 0;
    pool.admin = ctx.accounts.payer.key();
    pool.bump = ctx.bumps.pool;

    emit!(PoolInitialized {
        pool: pool.key(),
        token_a_mint: pool.token_a_mint,
        token_b_mint: pool.token_b_mint,
        lp_mint: pool.lp_mint,
    });

    Ok(())
}
