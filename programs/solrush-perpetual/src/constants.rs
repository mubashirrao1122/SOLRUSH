use anchor_lang::prelude::*;

/// Minimum leverage
pub const MIN_LEVERAGE: u8 = 2;

/// Maximum leverage
pub const MAX_LEVERAGE: u8 = 10;

/// Maintenance margin rate (10%)
pub const MAINTENANCE_MARGIN_RATE: u64 = 1000; // basis points

/// Liquidation fee (2%)
pub const LIQUIDATION_FEE: u64 = 200; // basis points

/// Position state seed
pub const POSITION_SEED: &[u8] = b"position";

/// Funding rate seed
pub const FUNDING_RATE_SEED: &[u8] = b"funding_rate";

/// Collateral vault seed
pub const COLLATERAL_VAULT_SEED: &[u8] = b"collateral_vault";
