use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use solrush_liquidity_pool::state::PoolState;
use crate::{constants::*, state::{RewardState, UserRewardAccount}, utils::*};

#[derive(Accounts)]
pub struct UpdateUserRewards<'info> {
    #[account(
        seeds = [REWARD_STATE_SEED],
        bump = reward_state.bump,
    )]
    pub reward_state: Account<'info, RewardState>,

    #[account(
        mut,
        seeds = [
            USER_REWARD_SEED,
            user.key().as_ref(),
            pool.key().as_ref(),
        ],
        bump = user_reward_account.bump,
    )]
    pub user_reward_account: Account<'info, UserRewardAccount>,

    pub pool: Account<'info, PoolState>,

    pub user_lp_token_account: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateUserRewards>) -> Result<()> {
    let reward_state = &ctx.accounts.reward_state;
    let user_reward = &mut ctx.accounts.user_reward_account;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    let time_elapsed = clock.unix_timestamp - user_reward.last_update_time;
    
    if time_elapsed > 0 && user_reward.lp_token_balance > 0 {
        let pending = calculate_rewards(
            user_reward.lp_token_balance,
            pool.lp_token_supply,
            time_elapsed,
            reward_state.reward_rate_per_second,
        )?;

        user_reward.earned_rewards = user_reward.earned_rewards
            .checked_add(pending)
            .unwrap_or(user_reward.earned_rewards);
    }

    user_reward.lp_token_balance = ctx.accounts.user_lp_token_account.amount;
    user_reward.last_update_time = clock.unix_timestamp;

    Ok(())
}
