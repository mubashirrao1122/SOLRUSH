use anchor_lang::prelude::*;
use crate::state::{PoolState, PoolInfo};

#[derive(Accounts)]
pub struct GetPoolInfo<'info> {
    #[account(
        seeds = [
            b"pool",
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump = pool.bump,
    )]
    pub pool: Account<'info, PoolState>,
}

pub fn handler(ctx: Context<GetPoolInfo>) -> Result<PoolInfo> {
    let pool = &ctx.accounts.pool;

    Ok(PoolInfo {
        token_a_reserve: pool.token_a_reserve,
        token_b_reserve: pool.token_b_reserve,
        lp_token_supply: pool.lp_token_supply,
        fee_rate: pool.fee_rate,
        tvl_token_a: pool.token_a_reserve,
        tvl_token_b: pool.token_b_reserve,
        is_paused: pool.is_paused,
    })
}
