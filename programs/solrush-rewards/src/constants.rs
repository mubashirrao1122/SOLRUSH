use anchor_lang::prelude::*;

/// Reward rate denominator (for precision)
pub const REWARD_PRECISION: u64 = 1_000_000_000;

/// Minimum reward rate per second
pub const MIN_REWARD_RATE: u64 = 1;

/// Maximum reward rate per second (prevent overflow)
pub const MAX_REWARD_RATE: u64 = 1_000_000_000_000;

/// Reward state seed
pub const REWARD_STATE_SEED: &[u8] = b"reward_state";

/// User reward seed
pub const USER_REWARD_SEED: &[u8] = b"user_reward";
