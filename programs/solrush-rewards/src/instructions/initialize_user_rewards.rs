use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::{constants::*, state::UserRewardAccount};

#[derive(Accounts)]
pub struct InitializeUserRewards<'info> {
    #[account(
        init,
        payer = user,
        space = UserRewardAccount::LEN,
        seeds = [
            USER_REWARD_SEED,
            user.key().as_ref(),
            pool.key().as_ref(),
        ],
        bump
    )]
    pub user_reward_account: Account<'info, UserRewardAccount>,

    /// CHECK: Pool account reference
    pub pool: UncheckedAccount<'info>,

    pub user_lp_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeUserRewards>) -> Result<()> {
    let user_reward = &mut ctx.accounts.user_reward_account;
    let clock = Clock::get()?;

    user_reward.bump = ctx.bumps.user_reward_account;
    user_reward.user = ctx.accounts.user.key();
    user_reward.pool = ctx.accounts.pool.key();
    user_reward.lp_token_balance = ctx.accounts.user_lp_token_account.amount;
    user_reward.earned_rewards = 0;
    user_reward.claimed_rewards = 0;
    user_reward.last_claim_time = clock.unix_timestamp;
    user_reward.last_update_time = clock.unix_timestamp;
    user_reward.reward_debt = 0;

    Ok(())
}
