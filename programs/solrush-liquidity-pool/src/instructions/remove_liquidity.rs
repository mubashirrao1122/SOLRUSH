use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
use crate::{constants::*, errors::PoolError, state::PoolState, utils::*};

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
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

    #[account(
        mut,
        constraint = lp_token_mint.key() == pool.lp_token_mint @ PoolError::InvalidPoolState,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_token_a.mint == pool.token_a_mint @ PoolError::InvalidTokenMint,
        constraint = user_token_a.owner == user.key() @ PoolError::Unauthorized,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.mint == pool.token_b_mint @ PoolError::InvalidTokenMint,
        constraint = user_token_b.owner == user.key() @ PoolError::Unauthorized,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_lp_token.mint == pool.lp_token_mint @ PoolError::InvalidTokenMint,
        constraint = user_lp_token.owner == user.key() @ PoolError::Unauthorized,
    )]
    pub user_lp_token: Account<'info, TokenAccount>,

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
    ctx: Context<RemoveLiquidity>,
    lp_token_amount: u64,
    min_token_a: u64,
    min_token_b: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    
    require!(!pool.is_paused, PoolError::PoolPaused);
    require!(lp_token_amount > 0, PoolError::InvalidAmount);
    require!(pool.lp_token_supply > 0, PoolError::InvalidPoolState);

    // Calculate token amounts to return
    let (token_a_amount, token_b_amount) = calculate_remove_liquidity(
        lp_token_amount,
        pool.token_a_reserve,
        pool.token_b_reserve,
        pool.lp_token_supply,
    )?;

    // Validate slippage
    validate_slippage(token_a_amount, token_a_amount, min_token_a)?;
    validate_slippage(token_b_amount, token_b_amount, min_token_b)?;

    // Burn LP tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_token_amount,
    )?;

    // Transfer tokens from vault to user
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
                from: ctx.accounts.token_a_vault.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer,
        ),
        token_a_amount,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_b_vault.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer,
        ),
        token_b_amount,
    )?;

    // Update pool state
    pool.token_a_reserve = pool.token_a_reserve
        .checked_sub(token_a_amount)
        .ok_or(PoolError::Underflow)?;
    pool.token_b_reserve = pool.token_b_reserve
        .checked_sub(token_b_amount)
        .ok_or(PoolError::Underflow)?;
    pool.lp_token_supply = pool.lp_token_supply
        .checked_sub(lp_token_amount)
        .ok_or(PoolError::Underflow)?;

    emit!(LiquidityRemoved {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        token_a_amount,
        token_b_amount,
        lp_tokens_burned: lp_token_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub lp_tokens_burned: u64,
    pub timestamp: i64,
}
