use anchor_lang::prelude::*;
use crate::{constants::*, state::AdminState};

#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(
        init,
        payer = admin,
        space = AdminState::LEN,
        seeds = [ADMIN_STATE_SEED],
        bump
    )]
    pub admin_state: Account<'info, AdminState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeAdmin>) -> Result<()> {
    let admin_state = &mut ctx.accounts.admin_state;
    let clock = Clock::get()?;

    admin_state.bump = ctx.bumps.admin_state;
    admin_state.admin = ctx.accounts.admin.key();
    admin_state.is_paused = false;
    admin_state.pause_reason = String::new();
    admin_state.last_action_time = clock.unix_timestamp;
    admin_state.fee_update_count = 0;

    emit!(AdminInitialized {
        admin: admin_state.admin,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct AdminInitialized {
    pub admin: Pubkey,
    pub timestamp: i64,
}
