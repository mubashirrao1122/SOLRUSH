use anchor_lang::prelude::*;
use crate::{constants::*, state::TokenState};

#[derive(Accounts)]
pub struct GetCirculatingSupply<'info> {
    #[account(
        seeds = [TOKEN_STATE_SEED],
        bump = token_state.bump,
    )]
    pub token_state: Account<'info, TokenState>,
}

pub fn handler(ctx: Context<GetCirculatingSupply>) -> Result<u64> {
    let token_state = &ctx.accounts.token_state;
    Ok(token_state.circulating_supply)
}
