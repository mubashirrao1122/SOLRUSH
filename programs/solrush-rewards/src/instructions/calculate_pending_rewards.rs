use anchor_lang::prelude::*;
use solrush_liquidity_pool::state::PoolState;
use crate::{constants::*, state::{RewardState, UserRewardAccount, RewardInfo}, utils::*};

#[derive(Accounts)]
pub struct CalculatePendingRewards<'info> {
    #[account(
        seeds = [REWARD_STATE_SEED],
        bump = reward_state.bump,
    )]
    pub reward_state: Account<'info, RewardState>,

    #[account(
        seeds = [
            USER_REWARD_SEED,
            user_reward_account.user.as_ref(),
            user_reward_account.pool.as_ref(),
        ],
        bump = user_reward_account.bump,
    )]
    pub user_reward_account: Account<'info, UserRewardAccount>,

    pub pool: Account<'info, PoolState>,
}

pub fn handler(ctx: Context<CalculatePendingRewards>) -> Result<RewardInfo> {
    let reward_state = &ctx.accounts.reward_state;
    let user_reward = &ctx.accounts.user_reward_account;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    let time_elapsed = clock.unix_timestamp - user_reward.last_update_time;
    
    let pending = if time_elapsed > 0 && user_reward.lp_token_balance > 0 {
        calculate_rewards(
            user_reward.lp_token_balance,
            pool.lp_token_supply,
            time_elapsed,
            reward_state.reward_rate_per_second,
        )?
    } else {
        0
    };

    Ok(RewardInfo {
        pending_rewards: pending,
        earned_rewards: user_reward.earned_rewards,
        claimed_rewards: user_reward.claimed_rewards,
        lp_token_balance: user_reward.lp_token_balance,
    })
}
