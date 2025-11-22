use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use crate::{constants::*, errors::PoolError, state::PoolState, utils::*};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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
    ctx: Context<AddLiquidity>,
    token_a_amount: u64,
    token_b_amount: u64,
    min_lp_tokens: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    
    require!(!pool.is_paused, PoolError::PoolPaused);
    require!(token_a_amount > 0 && token_b_amount > 0, PoolError::InvalidAmount);

    let lp_tokens = if pool.lp_token_supply == 0 {
        // Initial liquidity
        let lp = calculate_initial_lp_tokens(token_a_amount, token_b_amount)?;
        
        // Lock minimum liquidity
        require!(lp > MINIMUM_LIQUIDITY, PoolError::InvalidAmount);
        
        lp.checked_sub(MINIMUM_LIQUIDITY).ok_or(PoolError::Underflow)?
    } else {
        // Additional liquidity
        calculate_lp_tokens(
            token_a_amount,
            token_b_amount,
            pool.token_a_reserve,
            pool.token_b_reserve,
            pool.lp_token_supply,
        )?
    };

    // Validate slippage
    validate_slippage(lp_tokens, lp_tokens, min_lp_tokens)?;

    // Transfer tokens from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.token_a_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        token_a_amount,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.token_b_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        token_b_amount,
    )?;

    // Mint LP tokens to user
    let seeds = &[
        POOL_AUTHORITY_SEED,
        pool.token_a_mint.as_ref(),
        pool.token_b_mint.as_ref(),
        &[ctx.bumps.pool_authority],
    ];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                to: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer,
        ),
        lp_tokens,
    )?;

    // Update pool state
    pool.token_a_reserve = pool.token_a_reserve
        .checked_add(token_a_amount)
        .ok_or(PoolError::Overflow)?;
    pool.token_b_reserve = pool.token_b_reserve
        .checked_add(token_b_amount)
        .ok_or(PoolError::Overflow)?;
    
    if pool.lp_token_supply == 0 {
        pool.lp_token_supply = lp_tokens
            .checked_add(MINIMUM_LIQUIDITY)
            .ok_or(PoolError::Overflow)?;
    } else {
        pool.lp_token_supply = pool.lp_token_supply
            .checked_add(lp_tokens)
            .ok_or(PoolError::Overflow)?;
    }

    emit!(LiquidityAdded {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        token_a_amount,
        token_b_amount,
        lp_tokens_minted: lp_tokens,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub lp_tokens_minted: u64,
    pub timestamp: i64,
}
