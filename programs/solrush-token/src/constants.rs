use anchor_lang::prelude::*;

/// Total supply cap (1,000,000 tokens with 9 decimals)
pub const TOTAL_SUPPLY_CAP: u64 = 1_000_000_000_000_000;

/// Token decimals
pub const TOKEN_DECIMALS: u8 = 9;

/// Token name
pub const TOKEN_NAME: &str = "Rush Token";

/// Token symbol
pub const TOKEN_SYMBOL: &str = "RUSH";

/// Token metadata URI
pub const TOKEN_URI: &str = "https://solrush.io/metadata/rush.json";

/// Token state seed
pub const TOKEN_STATE_SEED: &[u8] = b"rush_token_state";

/// Mint authority seed
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";
