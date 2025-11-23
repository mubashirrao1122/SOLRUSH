use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::SwapError;

#[derive(Accounts)]
pub struct CreateDCAOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"pool", pool.trading_pair.seed()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        init,
        payer = user,
        space = DCAOrder::LEN,
        seeds = [
            b"dca_order",
            user.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub dca_order: Account<'info, DCAOrder>,

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
        token::authority = dca_order,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateDCAOrder>,
    trading_pair: TradingPair,
    order_side: OrderSide,
    amount_per_cycle: u64,
    total_cycles: u16,
    cycle_frequency: i64,
    slippage_tolerance: u16,
    min_price: u64,
    max_price: u64,
) -> Result<()> {
    require!(amount_per_cycle > 0, SwapError::InvalidAmount);
    require!(total_cycles > 0 && total_cycles <= 1000, SwapError::MaxCyclesExceeded);
    require!(cycle_frequency >= 60, SwapError::InvalidCycleFrequency); // At least 60 seconds
    require!(slippage_tolerance <= 10000, SwapError::InvalidSlippageTolerance);

    // Validate price range if specified
    if min_price > 0 && max_price > 0 {
        require!(min_price < max_price, SwapError::PriceOutOfRange);
    }

    let clock = Clock::get()?;
    let dca_order = &mut ctx.accounts.dca_order;

    // Calculate total amount needed
    let total_amount = (amount_per_cycle as u128)
        .checked_mul(total_cycles as u128)
        .ok_or(SwapError::CalculationOverflow)? as u64;

    // Transfer total amount to escrow
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.escrow_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        total_amount,
    )?;

    // Initialize DCA order
    dca_order.owner = ctx.accounts.user.key();
    dca_order.trading_pair = trading_pair;
    dca_order.order_side = order_side;
    dca_order.order_status = OrderStatus::Open;
    dca_order.amount_per_cycle = amount_per_cycle;
    dca_order.total_cycles = total_cycles;
    dca_order.cycles_executed = 0;
    dca_order.cycle_frequency = cycle_frequency;
    dca_order.last_execution = 0;
    dca_order.next_execution = clock.unix_timestamp + cycle_frequency;
    dca_order.slippage_tolerance = slippage_tolerance;
    dca_order.min_price = min_price;
    dca_order.max_price = max_price;
    dca_order.user_token_account = ctx.accounts.user_token_account.key();
    dca_order.escrow_token_account = ctx.accounts.escrow_token_account.key();
    dca_order.total_amount_in = 0;
    dca_order.total_amount_out = 0;
    dca_order.bump = ctx.bumps.dca_order;

    msg!(
        "DCA order created: {} cycles of {} tokens every {} seconds (price range: {} - {})",
        total_cycles,
        amount_per_cycle,
        cycle_frequency,
        min_price,
        max_price
    );

    Ok(())
}
