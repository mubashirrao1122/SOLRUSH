use anchor_lang::prelude::*;
use crate::{constants::*, errors::RewardError, state::RewardState};

#[derive(Accounts)]
pub struct UpdateRewardRate<'info> {
    #[account(
        mut,
        seeds = [REWARD_STATE_SEED],
        bump = reward_state.bump,
        constraint = reward_state.admin == admin.key() @ RewardError::Unauthorized,
    )]
    pub reward_state: Account<'info, RewardState>,

    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateRewardRate>, new_rate: u64) -> Result<()> {
    require!(
        new_rate >= MIN_REWARD_RATE && new_rate <= MAX_REWARD_RATE,
        RewardError::InvalidRewardRate
    );

    let reward_state = &mut ctx.accounts.reward_state;
    let old_rate = reward_state.reward_rate_per_second;

    reward_state.reward_rate_per_second = new_rate;
    reward_state.last_update_time = Clock::get()?.unix_timestamp;

    emit!(RewardRateUpdated {
        old_rate,
        new_rate,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct RewardRateUpdated {
    pub old_rate: u64,
    pub new_rate: u64,
    pub timestamp: i64,
}
