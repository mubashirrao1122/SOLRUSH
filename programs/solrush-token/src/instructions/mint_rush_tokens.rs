use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use crate::{constants::*, errors::TokenError, state::TokenState};

#[derive(Accounts)]
pub struct MintRushTokens<'info> {
    #[account(
        mut,
        seeds = [TOKEN_STATE_SEED],
        bump = token_state.bump,
        constraint = token_state.is_initialized @ TokenError::InvalidTokenState,
    )]
    pub token_state: Account<'info, TokenState>,

    #[account(
        mut,
        constraint = token_mint.key() == token_state.token_mint @ TokenError::InvalidTokenState,
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub recipient: Account<'info, TokenAccount>,

    /// CHECK: PDA mint authority
    #[account(
        seeds = [MINT_AUTHORITY_SEED],
        bump,
        constraint = mint_authority.key() == token_state.mint_authority @ TokenError::UnauthorizedMintAuthority,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// The authorized minter (rewards program, etc.)
    pub minter: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<MintRushTokens>, amount: u64) -> Result<()> {
    let token_state = &mut ctx.accounts.token_state;

    require!(amount > 0, TokenError::InvalidMintAmount);

    // Check supply cap
    let new_total = token_state.total_minted
        .checked_add(amount)
        .ok_or(TokenError::Overflow)?;

    require!(new_total <= token_state.supply_cap, TokenError::SupplyCapExceeded);

    // Mint tokens
    let seeds = &[
        MINT_AUTHORITY_SEED,
        &[ctx.bumps.mint_authority],
    ];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;

    // Update state
    token_state.total_minted = new_total;
    token_state.circulating_supply = token_state.circulating_supply
        .checked_add(amount)
        .ok_or(TokenError::Overflow)?;

    emit!(RushTokensMinted {
        recipient: ctx.accounts.recipient.key(),
        amount,
        total_minted: token_state.total_minted,
        remaining: token_state.supply_cap - token_state.total_minted,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct RushTokensMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub total_minted: u64,
    pub remaining: u64,
    pub timestamp: i64,
}
