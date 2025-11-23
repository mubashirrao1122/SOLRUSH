use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct CancelLimitOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"order_book", limit_order.trading_pair.seed()],
        bump = order_book.bump
    )]
    pub order_book: Account<'info, OrderBook>,

    #[account(
        mut,
        seeds = [
            b"limit_order",
            user.key().as_ref(),
            order_book.key().as_ref(),
            // Order index would be passed as remaining_accounts
        ],
        bump = limit_order.bump,
        constraint = limit_order.owner == user.key() @ SwapError::UnauthorizedOrderAccess,
        constraint = limit_order.order_status != OrderStatus::Filled @ SwapError::OrderAlreadyFilled,
        constraint = limit_order.order_status != OrderStatus::Cancelled @ SwapError::OrderAlreadyCancelled,
        close = user
    )]
    pub limit_order: Account<'info, LimitOrder>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = limit_order.escrow_token_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelLimitOrder>) -> Result<()> {
    let limit_order = &mut ctx.accounts.limit_order;
    let order_book = &mut ctx.accounts.order_book;

    // Calculate remaining amount
    let remaining_amount = limit_order.amount_in
        .checked_sub(limit_order.amount_filled)
        .ok_or(SwapError::MathError)?;

    if remaining_amount > 0 {
        // Return escrowed tokens to user
        let order_book_key = order_book.key();
        let seeds = &[
            b"limit_order",
            limit_order.owner.as_ref(),
            order_book_key.as_ref(),
            &[limit_order.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: limit_order.to_account_info(),
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
                    authority: limit_order.to_account_info(),
                },
                signer,
            ),
        )?;
    }

    // Update order status
    limit_order.order_status = OrderStatus::Cancelled;

    // Update order book counts
    match limit_order.order_side {
        OrderSide::Buy => {
            order_book.buy_orders_count = order_book.buy_orders_count.saturating_sub(1);
        }
        OrderSide::Sell => {
            order_book.sell_orders_count = order_book.sell_orders_count.saturating_sub(1);
        }
    }

    msg!("Limit order cancelled: {} returned to user", remaining_amount);

    Ok(())
}
