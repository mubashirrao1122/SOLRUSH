use anchor_lang::prelude::*;

/// Global reward state
#[account]
pub struct RewardState {
    pub bump: u8,
    pub admin: Pubkey,
    pub reward_rate_per_second: u64,
    pub total_rewards_distributed: u64,
    pub last_update_time: i64,
    pub is_active: bool,
}

impl RewardState {
    pub const LEN: usize = 8 + 1 + 32 + 8 + 8 + 8 + 1;
}

/// User reward tracking
#[account]
pub struct UserRewardAccount {
    pub bump: u8,
    pub user: Pubkey,
    pub pool: Pubkey,
    pub lp_token_balance: u64,
    pub earned_rewards: u64,
    pub claimed_rewards: u64,
    pub last_claim_time: i64,
    pub last_update_time: i64,
    pub reward_debt: u64,
}

impl UserRewardAccount {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8;
}

/// Reward information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardInfo {
    pub pending_rewards: u64,
    pub earned_rewards: u64,
    pub claimed_rewards: u64,
    pub lp_token_balance: u64,
}
