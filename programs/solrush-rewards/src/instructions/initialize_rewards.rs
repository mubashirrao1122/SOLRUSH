use anchor_lang::prelude::*;
use crate::{constants::*, errors::RewardError, state::RewardState};

#[derive(Accounts)]
pub struct InitializeRewards<'info> {
    #[account(
        init,
        payer = admin,
        space = RewardState::LEN,
        seeds = [REWARD_STATE_SEED],
        bump
    )]
    pub reward_state: Account<'info, RewardState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeRewards>, reward_rate_per_second: u64) -> Result<()> {
    require!(
        reward_rate_per_second >= MIN_REWARD_RATE && reward_rate_per_second <= MAX_REWARD_RATE,
        RewardError::InvalidRewardRate
    );

    let reward_state = &mut ctx.accounts.reward_state;
    let clock = Clock::get()?;

    reward_state.bump = ctx.bumps.reward_state;
    reward_state.admin = ctx.accounts.admin.key();
    reward_state.reward_rate_per_second = reward_rate_per_second;
    reward_state.total_rewards_distributed = 0;
    reward_state.last_update_time = clock.unix_timestamp;
    reward_state.is_active = true;

    emit!(RewardsInitialized {
        admin: reward_state.admin,
        reward_rate: reward_rate_per_second,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct RewardsInitialized {
    pub admin: Pubkey,
    pub reward_rate: u64,
    pub timestamp: i64,
}
