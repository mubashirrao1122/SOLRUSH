use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct PlaceLimitOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"pool", pool.trading_pair.seed()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        seeds = [b"order_book", pool.trading_pair.seed()],
        bump = order_book.bump,
        constraint = order_book.trading_pair == pool.trading_pair
    )]
    pub order_book: Account<'info, OrderBook>,

    #[account(
        init,
        payer = user,
        space = LimitOrder::LEN,
        seeds = [
            b"limit_order",
            user.key().as_ref(),
            order_book.key().as_ref(),
            &order_book.buy_orders_count.checked_add(order_book.sell_orders_count).unwrap().to_le_bytes()
        ],
        bump
    )]
    pub limit_order: Account<'info, LimitOrder>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: Token mint account
    pub token_mint: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        token::mint = token_mint,
        token::authority = limit_order,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<PlaceLimitOrder>,
    trading_pair: TradingPair,
    order_side: OrderSide,
    amount_in: u64,
    limit_price: u64,
    slippage_tolerance: u16,
    expires_at: i64,
) -> Result<()> {
    require!(amount_in > 0, SwapError::InvalidAmount);
    require!(limit_price > 0, SwapError::InvalidLimitPrice);
    require!(slippage_tolerance <= 10000, SwapError::InvalidSlippageTolerance);

    let clock = Clock::get()?;
    
    if expires_at > 0 {
        require!(expires_at > clock.unix_timestamp, SwapError::InvalidExpirationTime);
    }

    let order_book = &mut ctx.accounts.order_book;
    let limit_order = &mut ctx.accounts.limit_order;

    // Transfer tokens to escrow
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.escrow_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_in,
    )?;

    // Initialize limit order
    limit_order.owner = ctx.accounts.user.key();
    limit_order.order_book = order_book.key();
    limit_order.trading_pair = trading_pair;
    limit_order.order_side = order_side;
    limit_order.order_status = OrderStatus::Open;
    limit_order.amount_in = amount_in;
    limit_order.amount_filled = 0;
    limit_order.limit_price = limit_price;
    limit_order.slippage_tolerance = slippage_tolerance;
    limit_order.created_at = clock.unix_timestamp;
    limit_order.expires_at = expires_at;
    limit_order.user_token_account = ctx.accounts.user_token_account.key();
    limit_order.escrow_token_account = ctx.accounts.escrow_token_account.key();
    limit_order.bump = ctx.bumps.limit_order;

    // Update order book counts
    match order_side {
        OrderSide::Buy => {
            order_book.buy_orders_count = order_book.buy_orders_count
                .checked_add(1)
                .ok_or(SwapError::OrderBookFull)?;
        }
        OrderSide::Sell => {
            order_book.sell_orders_count = order_book.sell_orders_count
                .checked_add(1)
                .ok_or(SwapError::OrderBookFull)?;
        }
    }

    msg!("Limit order placed: {} {} at price {} (expires: {})",
        amount_in,
        match order_side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        },
        limit_price,
        expires_at
    );

    Ok(())
}
