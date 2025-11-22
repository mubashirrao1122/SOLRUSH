use anchor_lang::prelude::*;
use crate::{constants::*, errors::AdminError, state::AdminState};

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        seeds = [ADMIN_STATE_SEED],
        bump = admin_state.bump,
        constraint = admin_state.admin == current_admin.key() @ AdminError::Unauthorized,
    )]
    pub admin_state: Account<'info, AdminState>,

    pub current_admin: Signer<'info>,
}

pub fn handler(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
    let admin_state = &mut ctx.accounts.admin_state;
    let old_admin = admin_state.admin;

    admin_state.admin = new_admin;
    admin_state.last_action_time = Clock::get()?.unix_timestamp;

    emit!(AdminTransferred {
        old_admin,
        new_admin,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct AdminTransferred {
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
    pub timestamp: i64,
}
