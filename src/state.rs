use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    /// Primary key of the AMM and the mint of the token A
    pub token_a_mint: Pubkey,
    /// The mint of the token B
    pub token_b_mint: Pubkey,
    /// The vault holding token A
    pub vault_a: Pubkey,
    /// The vault holding token B
    pub vault_b: Pubkey,
    /// The mint for liquidity provider tokens
    pub lp_mint: Pubkey,
    /// Fee rate in basis points (e.g., 30 = 0.3%)
    pub fee_rate_bps: u16,
    /// Protocol fee rate in basis points
    pub protocol_fee_rate_bps: u16,
    /// The vault to receive protocol fees
    pub protocol_fee_vault: Pubkey,
    /// Total liquidity in the pool
    pub total_liquidity: u64,
    /// Admin public key for updating fees
    pub admin: Pubkey,
    /// Bump seed for the pool PDA
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8 + // discriminator
        32 + // token_a_mint
        32 + // token_b_mint
        32 + // vault_a
        32 + // vault_b
        32 + // lp_mint
        2 + // fee_rate_bps
        2 + // protocol_fee_rate_bps
        32 + // protocol_fee_vault
        8 + // total_liquidity
        32 + // admin
        1; // bump
}
