use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct CancelDCAOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = dca_order.owner == user.key() @ SwapError::UnauthorizedOrderAccess,
        constraint = dca_order.order_status != OrderStatus::Filled @ SwapError::OrderAlreadyFilled,
        constraint = dca_order.order_status != OrderStatus::Cancelled @ SwapError::OrderAlreadyCancelled,
        close = user
    )]
    pub dca_order: Account<'info, DCAOrder>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = dca_order.escrow_token_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelDCAOrder>) -> Result<()> {
    let dca_order = &mut ctx.accounts.dca_order;

    // Calculate remaining cycles
    let remaining_cycles = dca_order.total_cycles
        .checked_sub(dca_order.cycles_executed)
        .ok_or(SwapError::MathError)?;

    let remaining_amount = (dca_order.amount_per_cycle as u128)
        .checked_mul(remaining_cycles as u128)
        .ok_or(SwapError::CalculationOverflow)? as u64;

    if remaining_amount > 0 {
        // Return remaining escrowed tokens to user
        let seeds = &[
            b"dca_order",
            dca_order.owner.as_ref(),
            &dca_order.last_execution.to_le_bytes(),
            &[dca_order.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: dca_order.to_account_info(),
                },
                signer,
            ),
            remaining_amount,
        )?;

        // Close escrow account
        token::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                CloseAccount {
                    account: ctx.accounts.escrow_token_account.to_account_info(),
                    destination: ctx.accounts.user.to_account_info(),
                    authority: dca_order.to_account_info(),
                },
                signer,
            ),
        )?;
    }

    // Update order status
    dca_order.order_status = OrderStatus::Cancelled;

    msg!(
        "DCA order cancelled: {}/{} cycles completed, {} tokens returned",
        dca_order.cycles_executed,
        dca_order.total_cycles,
        remaining_amount
    );

    Ok(())
}
