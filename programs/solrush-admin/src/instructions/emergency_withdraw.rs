use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solrush_liquidity_pool::state::PoolState;
use crate::{constants::*, errors::AdminError, state::AdminState};

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        seeds = [ADMIN_STATE_SEED],
        bump = admin_state.bump,
        constraint = admin_state.admin == admin.key() @ AdminError::Unauthorized,
        constraint = admin_state.is_paused @ AdminError::TradingActive,
    )]
    pub admin_state: Account<'info, AdminState>,

    #[account(
        constraint = pool.is_paused @ AdminError::TradingActive,
    )]
    pub pool: Account<'info, PoolState>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,

    /// CHECK: Pool authority PDA
    pub pool_authority: UncheckedAccount<'info>,

    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<EmergencyWithdraw>) -> Result<()> {
    let amount = ctx.accounts.vault.amount;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
        ),
        amount,
    )?;

    emit!(EmergencyWithdrawal {
        admin: ctx.accounts.admin.key(),
        pool: ctx.accounts.pool.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct EmergencyWithdrawal {
    pub admin: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
