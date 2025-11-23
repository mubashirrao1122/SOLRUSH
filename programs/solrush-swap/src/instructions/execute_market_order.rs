use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct ExecuteMarketOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.trading_pair.seed()],
        bump = pool.bump,
        constraint = !pool.is_paused @ SwapError::PoolPaused
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        constraint = user_token_in.owner == user.key()
    )]
    pub user_token_in: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_out.owner == user.key()
    )]
    pub user_token_out: Account<'info, TokenAccount>,

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

    #[account(
        init,
        payer = user,
        space = MarketOrderRecord::LEN,
        seeds = [
            b"market_order",
            user.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub order_record: Account<'info, MarketOrderRecord>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ExecuteMarketOrder>,
    amount_in: u64,
    minimum_amount_out: u64,
    slippage_tolerance: u16,
    order_side: OrderSide,
) -> Result<()> {
    require!(amount_in > 0, SwapError::InvalidAmount);
    require!(slippage_tolerance <= 10000, SwapError::InvalidSlippageTolerance);

    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Determine input/output based on order side
    let (reserve_in, reserve_out, input_is_token_a) = match order_side {
        OrderSide::Buy => {
            // Buying token B with SOL (token A)
            (pool.reserve_a, pool.reserve_b, true)
        }
        OrderSide::Sell => {
            // Selling token B for SOL (token A)
            (pool.reserve_b, pool.reserve_a, false)
        }
    };

    require!(reserve_in > 0 && reserve_out > 0, SwapError::InsufficientLiquidity);

    // Calculate swap output using constant product formula
    // output = (reserveOut * amountIn * (10000 - fee)) / (reserveIn * 10000 + amountIn * (10000 - fee))
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
    require!(amount_out >= minimum_amount_out, SwapError::SlippageExceeded);

    // Calculate fee amount
    let fee_amount = amount_in
        .checked_mul(pool.fee_rate as u64)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(10000)
        .ok_or(SwapError::MathError)?;

    // Calculate price impact
    let expected_price = reserve_out
        .checked_mul(1_000_000_000)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(reserve_in)
        .ok_or(SwapError::MathError)?;
    
    let actual_price = amount_out
        .checked_mul(1_000_000_000)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(amount_in)
        .ok_or(SwapError::MathError)?;
    
    let price_impact = if expected_price > actual_price {
        ((expected_price - actual_price) as u128)
            .checked_mul(10000)
            .ok_or(SwapError::CalculationOverflow)?
            .checked_div(expected_price as u128)
            .ok_or(SwapError::MathError)? as u16
    } else {
        0
    };

    // Check slippage tolerance
    require!(
        price_impact <= slippage_tolerance,
        SwapError::SlippageExceeded
    );

    // Execute token transfers based on order side
    if input_is_token_a {
        // User sends token A, receives token B
        // Transfer token A from user to pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_in.to_account_info(),
                    to: ctx.accounts.pool_token_a_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        // Transfer token B from pool to user
        let seeds = &[
            b"pool",
            pool.trading_pair.seed(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_b_vault.to_account_info(),
                    to: ctx.accounts.user_token_out.to_account_info(),
                    authority: pool.to_account_info(),
                },
                signer,
            ),
            amount_out,
        )?;

        // Update reserves
        pool.reserve_a = pool.reserve_a.checked_add(amount_in).ok_or(SwapError::CalculationOverflow)?;
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).ok_or(SwapError::MathError)?;
        pool.total_fees_a = pool.total_fees_a.checked_add(fee_amount).ok_or(SwapError::CalculationOverflow)?;
    } else {
        // User sends token B, receives token A
        // Transfer token B from user to pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_in.to_account_info(),
                    to: ctx.accounts.pool_token_b_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        // Transfer token A from pool to user
        let seeds = &[
            b"pool",
            pool.trading_pair.seed(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_a_vault.to_account_info(),
                    to: ctx.accounts.user_token_out.to_account_info(),
                    authority: pool.to_account_info(),
                },
                signer,
            ),
            amount_out,
        )?;

        // Update reserves
        pool.reserve_b = pool.reserve_b.checked_add(amount_in).ok_or(SwapError::CalculationOverflow)?;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out).ok_or(SwapError::MathError)?;
        pool.total_fees_b = pool.total_fees_b.checked_add(fee_amount).ok_or(SwapError::CalculationOverflow)?;
    }

    pool.last_update_timestamp = clock.unix_timestamp;

    // Record the order execution
    let order_record = &mut ctx.accounts.order_record;
    order_record.owner = ctx.accounts.user.key();
    order_record.trading_pair = pool.trading_pair;
    order_record.order_side = order_side;
    order_record.amount_in = amount_in;
    order_record.amount_out = amount_out;
    order_record.fee_paid = fee_amount;
    order_record.price_impact = price_impact;
    order_record.executed_at = clock.unix_timestamp;
    order_record.tx_signature = [0u8; 64]; // Will be filled by client

    msg!("Market order executed: {} in -> {} out, fee: {}, impact: {}bps",
        amount_in, amount_out, fee_amount, price_impact);

    Ok(())
}
