use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, Burn};
use crate::state::Pool;
use crate::events::LiquidityRemoved;
use crate::errors::AmmError;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
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
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64,
) -> Result<()> {
    require!(lp_amount > 0, AmmError::InvalidAmount);

    let pool = &mut ctx.accounts.pool;
    let total_liquidity = pool.total_liquidity;
    let vault_a_amount = ctx.accounts.vault_a.amount;
    let vault_b_amount = ctx.accounts.vault_b.amount;

    require!(total_liquidity > 0, AmmError::InsufficientLiquidity);

    // Calculate amounts to return
    let amount_a = (lp_amount as u128)
        .checked_mul(vault_a_amount as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(total_liquidity as u128)
        .ok_or(AmmError::MathOverflow)? as u64;

    let amount_b = (lp_amount as u128)
        .checked_mul(vault_b_amount as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(total_liquidity as u128)
        .ok_or(AmmError::MathOverflow)? as u64;

    require!(amount_a > 0 && amount_b > 0, AmmError::MathOverflow);

    // Burn LP tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    // Transfer Token A to user
    let seeds = &[
        b"pool",
        pool.token_a_mint.as_ref(),
        pool.token_b_mint.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer,
        ),
        amount_a,
    )?;

    // Transfer Token B to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer,
        ),
        amount_b,
    )?;

    pool.total_liquidity = pool.total_liquidity.checked_sub(lp_amount).ok_or(AmmError::MathOverflow)?;

    emit!(LiquidityRemoved {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        amount_a,
        amount_b,
        liquidity_burned: lp_amount,
    });

    Ok(())
}
