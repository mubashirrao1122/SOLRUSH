use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
#[instruction(trading_pair: TradingPair)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = LiquidityPool::LEN,
        seeds = [b"pool", trading_pair.seed()],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = token_a_mint,
        token::authority = pool,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = token_b_mint,
        token::authority = pool,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = pool,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    trading_pair: TradingPair,
    fee_rate: u16,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.authority = ctx.accounts.authority.key();
    pool.trading_pair = trading_pair;
    pool.token_a_mint = ctx.accounts.token_a_mint.key();
    pool.token_b_mint = ctx.accounts.token_b_mint.key();
    pool.token_a_vault = ctx.accounts.token_a_vault.key();
    pool.token_b_vault = ctx.accounts.token_b_vault.key();
    pool.lp_token_mint = ctx.accounts.lp_token_mint.key();
    pool.reserve_a = 0;
    pool.reserve_b = 0;
    pool.lp_supply = 0;
    pool.fee_rate = fee_rate;
    pool.total_fees_a = 0;
    pool.total_fees_b = 0;
    pool.last_update_timestamp = Clock::get()?.unix_timestamp;
    pool.is_paused = false;
    pool.bump = ctx.bumps.pool;

    msg!("Pool initialized for {:?} with fee rate: {}bps", trading_pair, fee_rate);

    Ok(())
}
