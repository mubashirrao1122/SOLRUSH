use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{constants::*, errors::PoolError, state::PoolState};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = payer,
        space = PoolState::LEN,
        seeds = [
            b"pool",
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool: Account<'info, PoolState>,

    /// Token A mint
    pub token_a_mint: Account<'info, Mint>,

    /// Token B mint
    pub token_b_mint: Account<'info, Mint>,

    /// Token A vault
    #[account(
        init,
        payer = payer,
        token::mint = token_a_mint,
        token::authority = pool_authority,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    /// Token B vault
    #[account(
        init,
        payer = payer,
        token::mint = token_b_mint,
        token::authority = pool_authority,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    /// LP token mint
    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = pool_authority,
        seeds = [
            LP_TOKEN_SEED,
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref(),
        ],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,

    /// Pool authority PDA
    /// CHECK: PDA authority for the pool
    #[account(
        seeds = [
            POOL_AUTHORITY_SEED,
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializePool>, fee_rate: u16) -> Result<()> {
    require!(fee_rate <= MAX_FEE_RATE, PoolError::InvalidFeeRate);

    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    pool.bump = ctx.bumps.pool;
    pool.lp_bump = ctx.bumps.lp_token_mint;
    pool.token_a_mint = ctx.accounts.token_a_mint.key();
    pool.token_b_mint = ctx.accounts.token_b_mint.key();
    pool.token_a_vault = ctx.accounts.token_a_vault.key();
    pool.token_b_vault = ctx.accounts.token_b_vault.key();
    pool.lp_token_mint = ctx.accounts.lp_token_mint.key();
    pool.token_a_reserve = 0;
    pool.token_b_reserve = 0;
    pool.lp_token_supply = 0;
    pool.fee_rate = fee_rate;
    pool.fee_token_a = 0;
    pool.fee_token_b = 0;
    pool.is_paused = false;
    pool.created_at = clock.unix_timestamp;
    pool.last_swap_time = clock.unix_timestamp;

    emit!(PoolInitialized {
        pool: pool.key(),
        token_a_mint: pool.token_a_mint,
        token_b_mint: pool.token_b_mint,
        fee_rate,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub fee_rate: u16,
    pub timestamp: i64,
}
