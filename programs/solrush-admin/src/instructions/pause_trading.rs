use anchor_lang::prelude::*;
use solrush_liquidity_pool::state::PoolState;
use crate::{constants::*, errors::AdminError, state::AdminState};

#[derive(Accounts)]
pub struct PauseTrading<'info> {
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

pub fn handler(ctx: Context<PauseTrading>, reason: String) -> Result<()> {
    let admin_state = &mut ctx.accounts.admin_state;
    let pool = &mut ctx.accounts.pool;

    require!(!admin_state.is_paused, AdminError::AlreadyPaused);

    admin_state.is_paused = true;
    admin_state.pause_reason = reason.clone();
    admin_state.last_action_time = Clock::get()?.unix_timestamp;

    pool.is_paused = true;

    emit!(TradingPaused {
        admin: admin_state.admin,
        reason,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct TradingPaused {
    pub admin: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}
