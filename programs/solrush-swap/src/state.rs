use anchor_lang::prelude::*;

/// Swap quote information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub price_impact: u16, // in basis points
    pub minimum_received: u64,
}

/// Swap fee information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapFeeInfo {
    pub fee_rate: u16,
    pub total_fees_token_a: u64,
    pub total_fees_token_b: u64,
    pub lp_fee_share: u64, // percentage to LP providers
}

/// Swap route for multi-hop swaps (future enhancement)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapRoute {
    pub pools: Vec<Pubkey>,
    pub expected_output: u64,
    pub minimum_output: u64,
}
