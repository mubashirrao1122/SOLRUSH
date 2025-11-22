use anchor_lang::prelude::*;
use solrush_liquidity_pool::state::PoolState;
use crate::state::SwapFeeInfo;

#[derive(Accounts)]
pub struct GetSwapFeeInfo<'info> {
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

pub fn handler(ctx: Context<GetSwapFeeInfo>) -> Result<SwapFeeInfo> {
    let pool = &ctx.accounts.pool;

    Ok(SwapFeeInfo {
        fee_rate: pool.fee_rate,
        total_fees_token_a: pool.fee_token_a,
        total_fees_token_b: pool.fee_token_b,
        lp_fee_share: 10000, // 100% to LP providers
    })
}
