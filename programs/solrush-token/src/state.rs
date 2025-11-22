use anchor_lang::prelude::*;

/// Token state tracking
#[account]
pub struct TokenState {
    /// Bump seed
    pub bump: u8,
    
    /// Mint authority
    pub mint_authority: Pubkey,
    
    /// Token mint address
    pub token_mint: Pubkey,
    
    /// Total minted supply
    pub total_minted: u64,
    
    /// Supply cap
    pub supply_cap: u64,
    
    /// Circulating supply (total - locked)
    pub circulating_supply: u64,
    
    /// Locked tokens (for vesting, staking, etc.)
    pub locked_tokens: u64,
    
    /// Is initialized
    pub is_initialized: bool,
    
    /// Creation timestamp
    pub created_at: i64,
}

impl TokenState {
    pub const LEN: usize = 8 + // discriminator
        1 + // bump
        32 + // mint_authority
        32 + // token_mint
        8 + // total_minted
        8 + // supply_cap
        8 + // circulating_supply
        8 + // locked_tokens
        1 + // is_initialized
        8; // created_at
}

/// Supply information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SupplyInfo {
    pub total_minted: u64,
    pub supply_cap: u64,
    pub circulating_supply: u64,
    pub locked_tokens: u64,
    pub remaining_mintable: u64,
}
