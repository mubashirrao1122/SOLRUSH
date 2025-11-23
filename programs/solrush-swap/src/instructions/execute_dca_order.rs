use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct ExecuteDCAOrder<'info> {
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
        constraint = dca_order.order_status == OrderStatus::Open || dca_order.order_status == OrderStatus::PartiallyFilled @ SwapError::InvalidOrderStatus,
        constraint = dca_order.trading_pair == pool.trading_pair
    )]
    pub dca_order: Account<'info, DCAOrder>,

    /// CHECK: Owner of the DCA order
    pub order_owner: AccountInfo<'info>,

    #[account(
        mut,
        constraint = order_owner_token_account.owner == order_owner.key()
    )]
    pub order_owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = dca_order.escrow_token_account
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

pub fn handler(ctx: Context<ExecuteDCAOrder>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let dca_order = &mut ctx.accounts.dca_order;
    let clock = Clock::get()?;

    // Check if it's time to execute
    require!(
        clock.unix_timestamp >= dca_order.next_execution,
        SwapError::DCANotReady
    );

    // Check if all cycles are completed
    require!(
        dca_order.cycles_executed < dca_order.total_cycles,
        SwapError::DCACompleted
    );

    // Get current market price
    let current_price = match dca_order.order_side {
        OrderSide::Buy => {
            pool.reserve_b
                .checked_mul(1_000_000_000)
                .ok_or(SwapError::CalculationOverflow)?
                .checked_div(pool.reserve_a)
                .ok_or(SwapError::MathError)?
        }
        OrderSide::Sell => {
            pool.reserve_a
                .checked_mul(1_000_000_000)
                .ok_or(SwapError::CalculationOverflow)?
                .checked_div(pool.reserve_b)
                .ok_or(SwapError::MathError)?
        }
    };

    // Check price range if specified
    if dca_order.min_price > 0 {
        require!(
            current_price >= dca_order.min_price,
            SwapError::PriceOutOfRange
        );
    }
    if dca_order.max_price > 0 {
        require!(
            current_price <= dca_order.max_price,
            SwapError::PriceOutOfRange
        );
    }

    let amount_in = dca_order.amount_per_cycle;

    // Calculate swap output using constant product formula
    let (reserve_in, reserve_out, input_is_token_a) = match dca_order.order_side {
        OrderSide::Buy => (pool.reserve_a, pool.reserve_b, true),
        OrderSide::Sell => (pool.reserve_b, pool.reserve_a, false),
    };

    let fee_adjusted_input = amount_in
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

    // Check slippage
    let minimum_expected = amount_out
        .checked_mul(10000u64.checked_sub(dca_order.slippage_tolerance as u64).ok_or(SwapError::MathError)?)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(10000)
        .ok_or(SwapError::MathError)?;

    require!(amount_out >= minimum_expected, SwapError::SlippageExceeded);

    // Execute the swap
    let seeds = &[
        b"dca_order",
        dca_order.owner.as_ref(),
        &dca_order.last_execution.to_le_bytes(),
        &[dca_order.bump],
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
                    authority: dca_order.to_account_info(),
                },
                signer,
            ),
            amount_in,
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
        pool.reserve_a = pool.reserve_a.checked_add(amount_in).ok_or(SwapError::CalculationOverflow)?;
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).ok_or(SwapError::MathError)?;
    } else {
        // Transfer token B from escrow to pool
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_b_vault.to_account_info(),
                    authority: dca_order.to_account_info(),
                },
                signer,
            ),
            amount_in,
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
        pool.reserve_b = pool.reserve_b.checked_add(amount_in).ok_or(SwapError::CalculationOverflow)?;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out).ok_or(SwapError::MathError)?;
    }

    // Update DCA order
    dca_order.cycles_executed += 1;
    dca_order.last_execution = clock.unix_timestamp;
    dca_order.next_execution = clock.unix_timestamp + dca_order.cycle_frequency;
    dca_order.total_amount_in = dca_order.total_amount_in
        .checked_add(amount_in)
        .ok_or(SwapError::CalculationOverflow)?;
    dca_order.total_amount_out = dca_order.total_amount_out
        .checked_add(amount_out)
        .ok_or(SwapError::CalculationOverflow)?;

    // Update status
    if dca_order.cycles_executed == dca_order.total_cycles {
        dca_order.order_status = OrderStatus::Filled;
    } else {
        dca_order.order_status = OrderStatus::PartiallyFilled;
    }

    pool.last_update_timestamp = clock.unix_timestamp;

    msg!(
        "DCA cycle {}/{} executed: {} in -> {} out at price {} (avg: {})",
        dca_order.cycles_executed,
        dca_order.total_cycles,
        amount_in,
        amount_out,
        current_price,
        if dca_order.total_amount_in > 0 {
            dca_order.total_amount_out
                .checked_mul(1_000_000_000)
                .unwrap_or(0)
                .checked_div(dca_order.total_amount_in)
                .unwrap_or(0)
        } else {
            0
        }
    );

    Ok(())
}
