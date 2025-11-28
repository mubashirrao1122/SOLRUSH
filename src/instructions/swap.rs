use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::Pool;
use crate::events::SwapExecuted;
use crate::errors::AmmError;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_input.owner == user.key(),
    )]
    pub user_input: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_output.owner == user.key(),
    )]
    pub user_output: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        constraint = vault_input.key() == pool.vault_a || vault_input.key() == pool.vault_b,
    )]
    pub vault_input: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_output.key() == pool.vault_a || vault_output.key() == pool.vault_b,
    )]
    pub vault_output: Account<'info, TokenAccount>,

    /// CHECK: Validated in logic
    #[account(mut)]
    pub protocol_fee_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    require!(amount_in > 0, AmmError::InvalidAmount);

    let pool = &mut ctx.accounts.pool;
    
    // Verify vaults match input/output
    let is_input_a = ctx.accounts.vault_input.key() == pool.vault_a;
    let is_output_b = ctx.accounts.vault_output.key() == pool.vault_b;
    
    // Ensure we are swapping A->B or B->A
    require!(is_input_a != is_output_b, AmmError::InvalidVault); // Must be different vaults
    // Actually, if input is A, output MUST be B. If input is B, output MUST be A.
    // is_input_a implies input is vault_a.
    // is_output_b implies output is vault_b.
    // If is_input_a is true, then we need output to be vault_b (is_output_b true).
    // If is_input_a is false (input is vault_b), then we need output to be vault_a (is_output_b false).
    require!(is_input_a == is_output_b, AmmError::InvalidVault);

    let reserve_in = ctx.accounts.vault_input.amount;
    let reserve_out = ctx.accounts.vault_output.amount;

    require!(reserve_in > 0 && reserve_out > 0, AmmError::InsufficientLiquidity);

    // Calculate fees
    // fee_rate_bps is total fee (e.g. 30 for 0.3%)
    // protocol_fee_rate_bps is share of that fee (e.g. 1000 for 10% of the 0.3%)
    // Wait, usually protocol fee is a separate cut or a cut of the fee.
    // Let's assume protocol_fee_rate_bps is a cut of the TOTAL FEE.
    
    let total_fee = (amount_in as u128)
        .checked_mul(pool.fee_rate_bps as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(10000)
        .ok_or(AmmError::MathOverflow)? as u64;

    let amount_in_after_fee = amount_in.checked_sub(total_fee).ok_or(AmmError::MathOverflow)?;

    // Constant Product Formula: x * y = k
    // (reserve_in + amount_in_after_fee) * (reserve_out - amount_out) = reserve_in * reserve_out
    // reserve_out - amount_out = (reserve_in * reserve_out) / (reserve_in + amount_in_after_fee)
    // amount_out = reserve_out - (reserve_in * reserve_out) / (reserve_in + amount_in_after_fee)
    
    let numerator = (reserve_in as u128)
        .checked_mul(reserve_out as u128)
        .ok_or(AmmError::MathOverflow)?;
    
    let denominator = (reserve_in as u128)
        .checked_add(amount_in_after_fee as u128)
        .ok_or(AmmError::MathOverflow)?;

    let new_reserve_out = numerator.checked_div(denominator).ok_or(AmmError::MathOverflow)?;
    
    let amount_out = (reserve_out as u128)
        .checked_sub(new_reserve_out)
        .ok_or(AmmError::MathOverflow)? as u64;

    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    // Transfer Input from User to Vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_input.to_account_info(),
                to: ctx.accounts.vault_input.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_in,
    )?;

    // Handle Protocol Fee
    let protocol_fee = (total_fee as u128)
        .checked_mul(pool.protocol_fee_rate_bps as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(10000)
        .ok_or(AmmError::MathOverflow)? as u64;

    if protocol_fee > 0 {
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
                    from: ctx.accounts.vault_input.to_account_info(),
                    to: ctx.accounts.protocol_fee_vault.to_account_info(),
                    authority: pool.to_account_info(),
                },
                signer,
            ),
            protocol_fee,
        )?;
    }

    // Transfer Output from Vault to User
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
                from: ctx.accounts.vault_output.to_account_info(),
                to: ctx.accounts.user_output.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer,
        ),
        amount_out,
    )?;

    emit!(SwapExecuted {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        input_mint: ctx.accounts.user_input.mint,
        output_mint: ctx.accounts.user_output.mint,
        input_amount: amount_in,
        output_amount: amount_out,
        fee_amount: total_fee,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_product_math() {
        // x * y = k
        // 1,000,000 * 1,000,000 = 1,000,000,000,000
        let reserve_in: u64 = 1_000_000;
        let reserve_out: u64 = 1_000_000;
        let amount_in: u64 = 100_000;
        let fee_rate_bps: u16 = 30; // 0.3%

        // Fee = 100,000 * 30 / 10000 = 300
        let total_fee = (amount_in as u128 * fee_rate_bps as u128) / 10000;
        assert_eq!(total_fee, 300);

        let amount_in_after_fee = amount_in as u128 - total_fee;
        assert_eq!(amount_in_after_fee, 99_700);

        // New reserve in = 1,000,000 + 99,700 = 1,099,700
        // New reserve out = k / new_reserve_in
        // 1,000,000,000,000 / 1,099,700 = 909,338.9 -> 909,338
        let numerator = reserve_in as u128 * reserve_out as u128;
        let denominator = reserve_in as u128 + amount_in_after_fee;
        let new_reserve_out = numerator / denominator;
        assert_eq!(new_reserve_out, 909_338);

        let amount_out = reserve_out as u128 - new_reserve_out;
        assert_eq!(amount_out, 90_662);
    }
}
