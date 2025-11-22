use anchor_lang::prelude::*;

#[error_code]
pub enum PerpetualError {
    #[msg("Invalid leverage")]
    InvalidLeverage,
    
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    
    #[msg("Position undercollateralized")]
    Undercollateralized,
    
    #[msg("Position not liquidatable")]
    NotLiquidatable,
    
    #[msg("Invalid position size")]
    InvalidPositionSize,
    
    #[msg("Position already closed")]
    PositionClosed,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Math overflow")]
    Overflow,
    
    #[msg("Invalid price")]
    InvalidPrice,
    
    #[msg("Liquidation threshold not reached")]
    LiquidationThresholdNotReached,
}
