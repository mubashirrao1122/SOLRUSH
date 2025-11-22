use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
}

/// Position account
#[account]
pub struct Position {
    pub bump: u8,
    pub owner: Pubkey,
    pub pair: Pubkey,
    pub side: PositionSide,
    pub size: u64,
    pub leverage: u8,
    pub collateral: u64,
    pub entry_price: u64,
    pub liquidation_price: u64,
    pub take_profit: Option<u64>,
    pub stop_loss: Option<u64>,
    pub funding_index: i64,
    pub is_open: bool,
    pub opened_at: i64,
    pub closed_at: Option<i64>,
}

impl Position {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1 + 8 + 1 + 8 + 8 + 8 + 9 + 9 + 8 + 1 + 8 + 9;
}

/// Funding rate state
#[account]
pub struct FundingRate {
    pub bump: u8,
    pub pair: Pubkey,
    pub current_rate: i64,
    pub last_update: i64,
    pub cumulative_funding: i64,
}

impl FundingRate {
    pub const LEN: usize = 8 + 1 + 32 + 8 + 8 + 8;
}

/// PnL information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PnLInfo {
    pub unrealized_pnl: i64,
    pub realized_pnl: i64,
    pub funding_payment: i64,
    pub liquidation_price: u64,
    pub is_liquidatable: bool,
}
