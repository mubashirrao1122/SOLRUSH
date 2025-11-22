use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::{constants::*, errors::TokenError, state::TokenState};

#[derive(Accounts)]
pub struct InitializeRushToken<'info> {
    #[account(
        init,
        payer = payer,
        space = TokenState::LEN,
        seeds = [TOKEN_STATE_SEED],
        bump
    )]
    pub token_state: Account<'info, TokenState>,

    #[account(
        init,
        payer = payer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = mint_authority,
    )]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: PDA mint authority
    #[account(
        seeds = [MINT_AUTHORITY_SEED],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeRushToken>) -> Result<()> {
    let token_state = &mut ctx.accounts.token_state;
    let clock = Clock::get()?;

    require!(!token_state.is_initialized, TokenError::AlreadyInitialized);

    token_state.bump = ctx.bumps.token_state;
    token_state.mint_authority = ctx.accounts.mint_authority.key();
    token_state.token_mint = ctx.accounts.token_mint.key();
    token_state.total_minted = 0;
    token_state.supply_cap = TOTAL_SUPPLY_CAP;
    token_state.circulating_supply = 0;
    token_state.locked_tokens = 0;
    token_state.is_initialized = true;
    token_state.created_at = clock.unix_timestamp;

    emit!(RushTokenInitialized {
        token_mint: token_state.token_mint,
        mint_authority: token_state.mint_authority,
        supply_cap: token_state.supply_cap,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct RushTokenInitialized {
    pub token_mint: Pubkey,
    pub mint_authority: Pubkey,
    pub supply_cap: u64,
    pub timestamp: i64,
}
