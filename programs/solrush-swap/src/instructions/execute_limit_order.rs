use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct ExecuteLimitOrder<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.trading_pair.seed()],
        bump = pool.bump,
        constraint = !pool.is_paused @ SwapError::PoolPaused
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        seeds = [b"order_book", limit_order.trading_pair.seed()],
        bump = order_book.bump
    )]
    pub order_book: Account<'info, OrderBook>,

    #[account(
        mut,
        constraint = limit_order.order_status == OrderStatus::Open || limit_order.order_status == OrderStatus::PartiallyFilled @ SwapError::InvalidOrderStatus,
        constraint = limit_order.order_book == order_book.key()
    )]
    pub limit_order: Account<'info, LimitOrder>,

    /// CHECK: Owner of the limit order
    pub order_owner: AccountInfo<'info>,

    #[account(
        mut,
        constraint = order_owner_token_account.owner == order_owner.key()
    )]
    pub order_owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = limit_order.escrow_token_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.token_a_vault
    )]
    pub pool_token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.token_b_vault
    )]
    pub pool_token_b_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ExecuteLimitOrder>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let limit_order = &mut ctx.accounts.limit_order;
    let order_book = &mut ctx.accounts.order_book;
    let clock = Clock::get()?;

    // Check if order has expired
    if limit_order.expires_at > 0 && clock.unix_timestamp > limit_order.expires_at {
        limit_order.order_status = OrderStatus::Expired;
        return Err(SwapError::OrderExpired.into());
    }

    // Get current market price from pool
    let current_price = if limit_order.order_side == OrderSide::Buy {
        // Buying token B, price in token B per token A
        pool.reserve_b
            .checked_mul(1_000_000_000)
            .ok_or(SwapError::CalculationOverflow)?
            .checked_div(pool.reserve_a)
            .ok_or(SwapError::MathError)?
    } else {
        // Selling token B, price in token A per token B
        pool.reserve_a
            .checked_mul(1_000_000_000)
            .ok_or(SwapError::CalculationOverflow)?
            .checked_div(pool.reserve_b)
            .ok_or(SwapError::MathError)?
    };

    // Check if limit price is reached
    let price_met = match limit_order.order_side {
        OrderSide::Buy => current_price <= limit_order.limit_price,
        OrderSide::Sell => current_price >= limit_order.limit_price,
    };

    require!(price_met, SwapError::LimitPriceNotReached);

    // Calculate remaining amount to fill
    let remaining_amount = limit_order.amount_in
        .checked_sub(limit_order.amount_filled)
        .ok_or(SwapError::MathError)?;

    require!(remaining_amount > 0, SwapError::OrderAlreadyFilled);

    // Calculate swap output
    let (reserve_in, reserve_out, input_is_token_a) = match limit_order.order_side {
        OrderSide::Buy => (pool.reserve_a, pool.reserve_b, true),
        OrderSide::Sell => (pool.reserve_b, pool.reserve_a, false),
    };

    let fee_adjusted_input = remaining_amount
        .checked_mul(10000u64.checked_sub(pool.fee_rate as u64).ok_or(SwapError::MathError)?)
        .ok_or(SwapError::CalculationOverflow)?;
    
    let numerator = reserve_out
        .checked_mul(fee_adjusted_input)
        .ok_or(SwapError::CalculationOverflow)?;
    
    let denominator = reserve_in
        .checked_mul(10000)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_add(fee_adjusted_input)
        .ok_or(SwapError::CalculationOverflow)?;
    
    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(SwapError::ZeroOutput)?;

    require!(amount_out > 0, SwapError::ZeroOutput);

    // Calculate expected minimum with slippage
    let minimum_expected = amount_out
        .checked_mul(10000u64.checked_sub(limit_order.slippage_tolerance as u64).ok_or(SwapError::MathError)?)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(10000)
        .ok_or(SwapError::MathError)?;

    require!(amount_out >= minimum_expected, SwapError::SlippageExceeded);

    // Execute the swap
    let order_book_key = order_book.key();
    let seeds = &[
        b"limit_order",
        limit_order.owner.as_ref(),
        order_book_key.as_ref(),
        &[limit_order.bump],
    ];
    let signer = &[&seeds[..]];

    if input_is_token_a {
        // Transfer token A from escrow to pool
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_a_vault.to_account_info(),
                    authority: limit_order.to_account_info(),
                },
                signer,
            ),
            remaining_amount,
        )?;

        // Transfer token B from pool to order owner
        let pool_seeds = &[
            b"pool",
            pool.trading_pair.seed(),
            &[pool.bump],
        ];
        let pool_signer = &[&pool_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_b_vault.to_account_info(),
                    to: ctx.accounts.order_owner_token_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                pool_signer,
            ),
            amount_out,
        )?;

        // Update reserves
        pool.reserve_a = pool.reserve_a.checked_add(remaining_amount).ok_or(SwapError::CalculationOverflow)?;
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).ok_or(SwapError::MathError)?;
    } else {
        // Transfer token B from escrow to pool
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_b_vault.to_account_info(),
                    authority: limit_order.to_account_info(),
                },
                signer,
            ),
            remaining_amount,
        )?;

        // Transfer token A from pool to order owner
        let pool_seeds = &[
            b"pool",
            pool.trading_pair.seed(),
            &[pool.bump],
        ];
        let pool_signer = &[&pool_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_a_vault.to_account_info(),
                    to: ctx.accounts.order_owner_token_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                pool_signer,
            ),
            amount_out,
        )?;

        // Update reserves
        pool.reserve_b = pool.reserve_b.checked_add(remaining_amount).ok_or(SwapError::CalculationOverflow)?;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out).ok_or(SwapError::MathError)?;
    }

    // Update order status
    limit_order.amount_filled = limit_order.amount_in;
    limit_order.order_status = OrderStatus::Filled;
    pool.last_update_timestamp = clock.unix_timestamp;

    // Update order book volume
    order_book.total_volume = order_book.total_volume
        .checked_add(remaining_amount)
        .ok_or(SwapError::CalculationOverflow)?;

    msg!("Limit order executed: {} in -> {} out at market price {}", 
        remaining_amount, amount_out, current_price);

    Ok(())
}
