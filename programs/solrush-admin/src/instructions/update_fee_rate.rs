use anchor_lang::prelude::*;
use solrush_liquidity_pool::state::PoolState;
use crate::{constants::*, errors::AdminError, state::AdminState};

#[derive(Accounts)]
pub struct UpdateFeeRate<'info> {
    #[account(
        mut,
        seeds = [ADMIN_STATE_SEED],
        bump = admin_state.bump,
        constraint = admin_state.admin == admin.key() @ AdminError::Unauthorized,
    )]
    pub admin_state: Account<'info, AdminState>,

    #[account(mut)]
    pub pool: Account<'info, PoolState>,

    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateFeeRate>, new_rate: u16) -> Result<()> {
    require!(new_rate <= MAX_FEE_RATE, AdminError::InvalidFeeRate);

    let admin_state = &mut ctx.accounts.admin_state;
    let pool = &mut ctx.accounts.pool;
    let old_rate = pool.fee_rate;

    pool.fee_rate = new_rate;
    admin_state.fee_update_count += 1;
    admin_state.last_action_time = Clock::get()?.unix_timestamp;

    emit!(FeeRateUpdated {
        pool: pool.key(),
        old_rate,
        new_rate,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct FeeRateUpdated {
    pub pool: Pubkey,
    pub old_rate: u16,
    pub new_rate: u16,
    pub timestamp: i64,
}
