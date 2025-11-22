use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{errors::PerpetualError, state::Position};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(
        mut,
        constraint = position.owner == user.key() @ PerpetualError::Unauthorized,
        constraint = position.is_open @ PerpetualError::PositionClosed,
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: Vault authority PDA
    pub vault_authority: UncheckedAccount<'info>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClosePosition>) -> Result<()> {
    let position = &mut ctx.accounts.position;

    // Return collateral to user
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_collateral_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
        ),
        position.collateral,
    )?;

    position.is_open = false;
    position.closed_at = Some(Clock::get()?.unix_timestamp);

    emit!(PositionClosed {
        position: position.key(),
        owner: position.owner,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PositionClosed {
    pub position: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}
