use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{constants::*, errors::PoolError, state::PoolState, utils::*};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        mut,
        seeds = [
            b"pool",
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump = pool.bump,
    )]
    pub pool: Account<'info, PoolState>,

    #[account(
        mut,
        constraint = token_a_vault.key() == pool.token_a_vault @ PoolError::InvalidPoolState,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_b_vault.key() == pool.token_b_vault @ PoolError::InvalidPoolState,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_input_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_output_token: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
        seeds = [
            POOL_AUTHORITY_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<Swap>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    
    require!(!pool.is_paused, PoolError::PoolPaused);
    require!(amount_in > 0, PoolError::InvalidAmount);

    // Determine swap direction
    let is_a_to_b = ctx.accounts.user_input_token.mint == pool.token_a_mint;
    
    require!(
        is_a_to_b || ctx.accounts.user_input_token.mint == pool.token_b_mint,
        PoolError::InvalidTokenMint
    );

    let (input_reserve, output_reserve) = if is_a_to_b {
        (pool.token_a_reserve, pool.token_b_reserve)
    } else {
        (pool.token_b_reserve, pool.token_a_reserve)
    };

    // Calculate output amount
    let amount_out = calculate_swap_output(
        amount_in,
        input_reserve,
        output_reserve,
        pool.fee_rate,
    )?;

    // Validate slippage
    require!(amount_out >= minimum_amount_out, PoolError::SlippageExceeded);

    // Calculate fee
    let fee_amount = calculate_fee_amount(amount_in, pool.fee_rate)?;

    // Determine vault addresses
    let (input_vault, output_vault) = if is_a_to_b {
        (
            ctx.accounts.token_a_vault.to_account_info(),
            ctx.accounts.token_b_vault.to_account_info(),
        )
    } else {
        (
            ctx.accounts.token_b_vault.to_account_info(),
            ctx.accounts.token_a_vault.to_account_info(),
        )
    };

    // Transfer input tokens from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_input_token.to_account_info(),
                to: input_vault.clone(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_in,
    )?;

    // Transfer output tokens from vault to user
    let seeds = &[
        POOL_AUTHORITY_SEED,
        pool.token_a_mint.as_ref(),
        pool.token_b_mint.as_ref(),
        &[ctx.bumps.pool_authority],
    ];
    let signer = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: output_vault,
                to: ctx.accounts.user_output_token.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer,
        ),
        amount_out,
    )?;

    // Update pool reserves
    if is_a_to_b {
        pool.token_a_reserve = pool.token_a_reserve
            .checked_add(amount_in)
            .ok_or(PoolError::Overflow)?;
        pool.token_b_reserve = pool.token_b_reserve
            .checked_sub(amount_out)
            .ok_or(PoolError::Underflow)?;
        pool.fee_token_a = pool.fee_token_a
            .checked_add(fee_amount)
            .ok_or(PoolError::Overflow)?;
    } else {
        pool.token_b_reserve = pool.token_b_reserve
            .checked_add(amount_in)
            .ok_or(PoolError::Overflow)?;
        pool.token_a_reserve = pool.token_a_reserve
            .checked_sub(amount_out)
            .ok_or(PoolError::Underflow)?;
        pool.fee_token_b = pool.fee_token_b
            .checked_add(fee_amount)
            .ok_or(PoolError::Overflow)?;
    }

    let clock = Clock::get()?;
    pool.last_swap_time = clock.unix_timestamp;

    emit!(SwapExecuted {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        token_in: ctx.accounts.user_input_token.mint,
        token_out: ctx.accounts.user_output_token.mint,
        amount_in,
        amount_out,
        fee_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub timestamp: i64,
}
