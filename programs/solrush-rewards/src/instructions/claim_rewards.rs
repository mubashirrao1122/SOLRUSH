use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solrush_token::{
    program::SolrushToken,
    cpi::accounts::MintRushTokens,
    cpi::mint_rush_tokens,
};
use crate::{constants::*, errors::RewardError, state::{RewardState, UserRewardAccount}};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [REWARD_STATE_SEED],
        bump = reward_state.bump,
    )]
    pub reward_state: Account<'info, RewardState>,

    #[account(
        mut,
        seeds = [
            USER_REWARD_SEED,
            user.key().as_ref(),
            user_reward_account.pool.as_ref(),
        ],
        bump = user_reward_account.bump,
    )]
    pub user_reward_account: Account<'info, UserRewardAccount>,

    #[account(mut)]
    pub rush_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_rush_token_account: Account<'info, TokenAccount>,

    /// CHECK: Mint authority from token program
    pub mint_authority: UncheckedAccount<'info>,

    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub rush_token_program: Program<'info, SolrushToken>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let user_reward = &mut ctx.accounts.user_reward_account;
    let reward_state = &mut ctx.accounts.reward_state;
    
    let claimable = user_reward.earned_rewards
        .checked_sub(user_reward.claimed_rewards)
        .ok_or(RewardError::MathError)?;
    
    require!(claimable > 0, RewardError::NoRewardsToClaim);

    // Mint rewards through token program
    let cpi_accounts = MintRushTokens {
        token_state: ctx.accounts.reward_state.to_account_info(),
        token_mint: ctx.accounts.rush_token_mint.to_account_info(),
        recipient: ctx.accounts.user_rush_token_account.to_account_info(),
        mint_authority: ctx.accounts.mint_authority.to_account_info(),
        minter: ctx.accounts.user.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let seeds = &[REWARD_STATE_SEED, &[reward_state.bump]];
    let signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.rush_token_program.to_account_info(),
        cpi_accounts,
        signer,
    );

    mint_rush_tokens(cpi_ctx, claimable)?;

    user_reward.claimed_rewards = user_reward.claimed_rewards
        .checked_add(claimable)
        .ok_or(RewardError::CalculationOverflow)?;
    user_reward.last_claim_time = Clock::get()?.unix_timestamp;

    reward_state.total_rewards_distributed = reward_state.total_rewards_distributed
        .checked_add(claimable)
        .ok_or(RewardError::CalculationOverflow)?;

    emit!(RewardsClaimed {
        user: ctx.accounts.user.key(),
        amount: claimable,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct RewardsClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
