use anchor_lang::prelude::*;

/// Maximum fee rate in basis points (1% = 100 bp)
pub const MAX_FEE_RATE: u16 = 100;

/// Default fee rate (0.3% = 30 bp)
pub const DEFAULT_FEE_RATE: u16 = 30;

/// Basis points denominator
pub const BASIS_POINTS: u64 = 10000;

/// Minimum liquidity locked forever in the pool
pub const MINIMUM_LIQUIDITY: u64 = 1000;

/// Pool authority seed
pub const POOL_AUTHORITY_SEED: &[u8] = b"pool_authority";

/// LP token mint seed
pub const LP_TOKEN_SEED: &[u8] = b"lp_token";

/// Maximum slippage tolerance (10%)
pub const MAX_SLIPPAGE: u64 = 1000;
