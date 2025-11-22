use anchor_lang::prelude::*;

declare_id!("SRReward111111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod solrush_rewards {
    use super::*;

    /// Initialize reward distribution system
    pub fn initialize_rewards(
        ctx: Context<InitializeRewards>,
        reward_rate_per_second: u64,
    ) -> Result<()> {
        instructions::initialize_rewards::handler(ctx, reward_rate_per_second)
    }

    /// Initialize user reward account for a pool
    pub fn initialize_user_rewards(ctx: Context<InitializeUserRewards>) -> Result<()> {
        instructions::initialize_user_rewards::handler(ctx)
    }

    /// Update user rewards based on liquidity changes
    pub fn update_user_rewards(ctx: Context<UpdateUserRewards>) -> Result<()> {
        instructions::update_user_rewards::handler(ctx)
    }

    /// Claim accumulated rewards
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards::handler(ctx)
    }

    /// Calculate pending rewards (view function)
    pub fn calculate_pending_rewards(
        ctx: Context<CalculatePendingRewards>,
    ) -> Result<RewardInfo> {
        instructions::calculate_pending_rewards::handler(ctx)
    }

    /// Update reward rate (admin only)
    pub fn update_reward_rate(
        ctx: Context<UpdateRewardRate>,
        new_rate: u64,
    ) -> Result<()> {
        instructions::update_reward_rate::handler(ctx, new_rate)
    }
}
