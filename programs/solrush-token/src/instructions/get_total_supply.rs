use anchor_lang::prelude::*;
use crate::{constants::*, state::{TokenState, SupplyInfo}};

#[derive(Accounts)]
pub struct GetTotalSupply<'info> {
    #[account(
        seeds = [TOKEN_STATE_SEED],
        bump = token_state.bump,
    )]
    pub token_state: Account<'info, TokenState>,
}

pub fn handler(ctx: Context<GetTotalSupply>) -> Result<SupplyInfo> {
    let token_state = &ctx.accounts.token_state;

    let remaining_mintable = token_state.supply_cap
        .checked_sub(token_state.total_minted)
        .unwrap_or(0);

    Ok(SupplyInfo {
        total_minted: token_state.total_minted,
        supply_cap: token_state.supply_cap,
        circulating_supply: token_state.circulating_supply,
        locked_tokens: token_state.locked_tokens,
        remaining_mintable,
    })
}
