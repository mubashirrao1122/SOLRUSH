use anchor_lang::prelude::*;

/// Pool state account
#[account]
pub struct PoolState {
    /// Bump seed for PDA
    pub bump: u8,
    
    /// LP token mint bump
    pub lp_bump: u8,
    
    /// Token A mint address
    pub token_a_mint: Pubkey,
    
    /// Token B mint address
    pub token_b_mint: Pubkey,
    
    /// Token A vault
    pub token_a_vault: Pubkey,
    
    /// Token B vault
    pub token_b_vault: Pubkey,
    
    /// LP token mint
    pub lp_token_mint: Pubkey,
    
    /// Token A reserve balance
    pub token_a_reserve: u64,
    
    /// Token B reserve balance
    pub token_b_reserve: u64,
    
    /// Total LP token supply
    pub lp_token_supply: u64,
    
    /// Fee rate in basis points
    pub fee_rate: u16,
    
    /// Accumulated fees in token A
    pub fee_token_a: u64,
    
    /// Accumulated fees in token B
    pub fee_token_b: u64,
    
    /// Is pool paused
    pub is_paused: bool,
    
    /// Pool creation timestamp
    pub created_at: i64,
    
    /// Last swap timestamp
    pub last_swap_time: i64,
}

impl PoolState {
    pub const LEN: usize = 8 + // discriminator
        1 + // bump
        1 + // lp_bump
        32 + // token_a_mint
        32 + // token_b_mint
        32 + // token_a_vault
        32 + // token_b_vault
        32 + // lp_token_mint
        8 + // token_a_reserve
        8 + // token_b_reserve
        8 + // lp_token_supply
        2 + // fee_rate
        8 + // fee_token_a
        8 + // fee_token_b
        1 + // is_paused
        8 + // created_at
        8; // last_swap_time
}

/// Pool information returned by get_pool_info
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PoolInfo {
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub lp_token_supply: u64,
    pub fee_rate: u16,
    pub tvl_token_a: u64,
    pub tvl_token_b: u64,
    pub is_paused: bool,
}
