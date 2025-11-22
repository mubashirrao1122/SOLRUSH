use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use solrush_liquidity_pool::{
    program::SolrushLiquidityPool,
    cpi::accounts::Swap,
    cpi::swap,
    state::PoolState,
};
use crate::{errors::SwapError, utils::*};

#[derive(Accounts)]
pub struct ExecuteSwap<'info> {
    #[account(
        mut,
        seeds = [
            b"pool",
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump = pool.bump,
        seeds::program = liquidity_pool_program.key()
    )]
    pub pool: Account<'info, PoolState>,

    #[account(
        mut,
        constraint = token_a_vault.key() == pool.token_a_vault
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_b_vault.key() == pool.token_b_vault
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_input_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_output_token: Account<'info, TokenAccount>,

    /// CHECK: PDA authority validated by pool program
    pub pool_authority: UncheckedAccount<'info>,

    pub user: Signer<'info>,
    
    pub liquidity_pool_program: Program<'info, SolrushLiquidityPool>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<ExecuteSwap>,
    amount_in: u64,
    minimum_amount_out: u64,
    slippage_tolerance: u16,
) -> Result<()> {
    let pool = &ctx.accounts.pool;
    
    require!(!pool.is_paused, SwapError::PoolPaused);
    require!(amount_in > 0, SwapError::InvalidAmount);

    // Determine swap direction
    let is_a_to_b = ctx.accounts.user_input_token.mint == pool.token_a_mint;
    
    require!(
        is_a_to_b || ctx.accounts.user_input_token.mint == pool.token_b_mint,
        SwapError::InvalidTokenPair
    );

    let (reserve_in, reserve_out) = if is_a_to_b {
        (pool.token_a_reserve, pool.token_b_reserve)
    } else {
        (pool.token_b_reserve, pool.token_a_reserve)
    };

    // Calculate expected output
    let (expected_output, fee_amount) = calculate_swap_with_fee(
        amount_in,
        reserve_in,
        reserve_out,
        pool.fee_rate,
    )?;

    // Validate slippage
    validate_slippage(expected_output, minimum_amount_out, slippage_tolerance)?;

    // Calculate price impact
    let price_impact = calculate_price_impact(
        amount_in,
        expected_output,
        reserve_in,
        reserve_out,
    )?;

    // Validate price impact
    validate_price_impact(price_impact)?;

    // Execute swap through liquidity pool program
    let cpi_accounts = Swap {
        pool: ctx.accounts.pool.to_account_info(),
        token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
        token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
        user_input_token: ctx.accounts.user_input_token.to_account_info(),
        user_output_token: ctx.accounts.user_output_token.to_account_info(),
        pool_authority: ctx.accounts.pool_authority.to_account_info(),
        user: ctx.accounts.user.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.liquidity_pool_program.to_account_info(),
        cpi_accounts,
    );

    swap(cpi_ctx, amount_in, minimum_amount_out)?;

    emit!(SwapCompleted {
        user: ctx.accounts.user.key(),
        pool: pool.key(),
        token_in: ctx.accounts.user_input_token.mint,
        token_out: ctx.accounts.user_output_token.mint,
        amount_in,
        amount_out: expected_output,
        fee_amount,
        price_impact,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct SwapCompleted {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub price_impact: u16,
    pub timestamp: i64,
}
