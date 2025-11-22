use anchor_lang::prelude::*;
use solrush_liquidity_pool::state::PoolState;
use crate::{constants::*, errors::AdminError, state::AdminState};

#[derive(Accounts)]
pub struct ResumeTrading<'info> {
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

pub fn handler(ctx: Context<ResumeTrading>) -> Result<()> {
    let admin_state = &mut ctx.accounts.admin_state;
    let pool = &mut ctx.accounts.pool;

    require!(admin_state.is_paused, AdminError::NotPaused);

    admin_state.is_paused = false;
    admin_state.pause_reason = String::new();
    admin_state.last_action_time = Clock::get()?.unix_timestamp;

    pool.is_paused = false;

    emit!(TradingResumed {
        admin: admin_state.admin,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct TradingResumed {
    pub admin: Pubkey,
    pub timestamp: i64,
}
