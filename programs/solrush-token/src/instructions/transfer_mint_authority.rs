use anchor_lang::prelude::*;
use crate::{constants::*, errors::TokenError, state::TokenState};

#[derive(Accounts)]
pub struct TransferMintAuthority<'info> {
    #[account(
        mut,
        seeds = [TOKEN_STATE_SEED],
        bump = token_state.bump,
        constraint = token_state.is_initialized @ TokenError::InvalidTokenState,
    )]
    pub token_state: Account<'info, TokenState>,

    /// Current authority
    #[account(
        constraint = current_authority.key() == token_state.mint_authority @ TokenError::UnauthorizedMintAuthority,
    )]
    pub current_authority: Signer<'info>,
}

pub fn handler(ctx: Context<TransferMintAuthority>, new_authority: Pubkey) -> Result<()> {
    let token_state = &mut ctx.accounts.token_state;
    let old_authority = token_state.mint_authority;

    token_state.mint_authority = new_authority;

    emit!(MintAuthorityTransferred {
        old_authority,
        new_authority,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct MintAuthorityTransferred {
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub timestamp: i64,
}
