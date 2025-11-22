use anchor_lang::prelude::*;
use crate::{constants::*, state::FundingRate};

#[derive(Accounts)]
pub struct UpdateFundingRate<'info> {
    #[account(
        mut,
        seeds = [FUNDING_RATE_SEED, funding_rate.pair.as_ref()],
        bump = funding_rate.bump,
    )]
    pub funding_rate: Account<'info, FundingRate>,

    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateFundingRate>, new_rate: i64) -> Result<()> {
    let funding_rate = &mut ctx.accounts.funding_rate;
    let clock = Clock::get()?;

    funding_rate.current_rate = new_rate;
    funding_rate.last_update = clock.unix_timestamp;
    funding_rate.cumulative_funding = funding_rate.cumulative_funding
        .checked_add(new_rate)
        .unwrap_or(funding_rate.cumulative_funding);

    emit!(FundingRateUpdated {
        pair: funding_rate.pair,
        new_rate,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct FundingRateUpdated {
    pub pair: Pubkey,
    pub new_rate: i64,
    pub timestamp: i64,
}
