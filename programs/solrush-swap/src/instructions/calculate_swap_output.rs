use anchor_lang::prelude::*;
use solrush_liquidity_pool::state::PoolState;
use crate::{state::SwapQuote, utils::*};

#[derive(Accounts)]
pub struct CalculateSwapOutput<'info> {
    #[account(
        seeds = [
            b"pool",
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump = pool.bump,
        seeds::program = solrush_liquidity_pool::ID
    )]
    pub pool: Account<'info, PoolState>,
}

pub fn handler(
    ctx: Context<CalculateSwapOutput>,
    amount_in: u64,
    input_is_token_a: bool,
) -> Result<SwapQuote> {
    let pool = &ctx.accounts.pool;

    let (reserve_in, reserve_out) = if input_is_token_a {
        (pool.token_a_reserve, pool.token_b_reserve)
    } else {
        (pool.token_b_reserve, pool.token_a_reserve)
    };

    let (amount_out, fee_amount) = calculate_swap_with_fee(
        amount_in,
        reserve_in,
        reserve_out,
        pool.fee_rate,
    )?;

    let price_impact = calculate_price_impact(
        amount_in,
        amount_out,
        reserve_in,
        reserve_out,
    )?;

    // Calculate minimum with 0.5% slippage
    let minimum_received = (amount_out as u128)
        .checked_mul(9950u128)
        .unwrap()
        .checked_div(10000u128)
        .unwrap() as u64;

    Ok(SwapQuote {
        amount_in,
        amount_out,
        fee_amount,
        price_impact,
        minimum_received,
    })
}
