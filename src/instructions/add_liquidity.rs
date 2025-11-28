use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo};
use crate::state::Pool;
use crate::events::LiquidityAdded;
use crate::errors::AmmError;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_a.mint == pool.token_a_mint,
        constraint = user_token_a.owner == user.key(),
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.mint == pool.token_b_mint,
        constraint = user_token_b.owner == user.key(),
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_lp.mint == pool.lp_mint,
        constraint = user_lp.owner == user.key(),
    )]
    pub user_lp: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"vault_a", pool.key().as_ref()],
        bump,
        constraint = vault_a.key() == pool.vault_a,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_b", pool.key().as_ref()],
        bump,
        constraint = vault_b.key() == pool.vault_b,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp_mint", pool.key().as_ref()],
        bump,
        constraint = lp_mint.key() == pool.lp_mint,
    )]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, AmmError::InvalidAmount);

    let pool = &mut ctx.accounts.pool;
    let vault_a_amount = ctx.accounts.vault_a.amount;
    let vault_b_amount = ctx.accounts.vault_b.amount;
    let total_liquidity = pool.total_liquidity;

    let liquidity_to_mint;

    if total_liquidity == 0 {
        // Initial liquidity
        // Geometric mean of amounts: sqrt(a * b)
        // Using u128 to prevent overflow before sqrt
        let product = (amount_a as u128).checked_mul(amount_b as u128).ok_or(AmmError::MathOverflow)?;
        liquidity_to_mint = (product as f64).sqrt() as u64;
    } else {
        // Subsequent liquidity
        // min(amount_a * total_liquidity / reserve_a, amount_b * total_liquidity / reserve_b)
        let share_a = (amount_a as u128)
            .checked_mul(total_liquidity as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(vault_a_amount as u128)
            .ok_or(AmmError::MathOverflow)?;

        let share_b = (amount_b as u128)
            .checked_mul(total_liquidity as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(vault_b_amount as u128)
            .ok_or(AmmError::MathOverflow)?;

        liquidity_to_mint = std::cmp::min(share_a, share_b) as u64;
    }

    require!(liquidity_to_mint > 0, AmmError::InsufficientLiquidity);

    // Transfer Token A
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_a,
    )?;

    // Transfer Token B
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_b,
    )?;

    // Mint LP Tokens
    let seeds = &[
        b"pool",
        pool.token_a_mint.as_ref(),
        pool.token_b_mint.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer,
        ),
        liquidity_to_mint,
    )?;

    pool.total_liquidity = pool.total_liquidity.checked_add(liquidity_to_mint).ok_or(AmmError::MathOverflow)?;

    emit!(LiquidityAdded {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        amount_a,
        amount_b,
        liquidity_minted: liquidity_to_mint,
    });

    Ok(())
}
