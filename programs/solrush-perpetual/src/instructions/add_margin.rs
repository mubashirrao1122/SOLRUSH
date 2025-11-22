use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{errors::PerpetualError, state::Position};

#[derive(Accounts)]
pub struct AddMargin<'info> {
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

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AddMargin>, additional_collateral: u64) -> Result<()> {
    require!(additional_collateral > 0, PerpetualError::InsufficientCollateral);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_collateral_account.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        additional_collateral,
    )?;

    let position = &mut ctx.accounts.position;
    position.collateral = position.collateral
        .checked_add(additional_collateral)
        .ok_or(PerpetualError::Overflow)?;

    emit!(MarginAdded {
        position: position.key(),
        amount: additional_collateral,
        new_collateral: position.collateral,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct MarginAdded {
    pub position: Pubkey,
    pub amount: u64,
    pub new_collateral: u64,
    pub timestamp: i64,
}
