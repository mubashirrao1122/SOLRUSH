use anchor_lang::prelude::*;

#[error_code]
pub enum AdminError {
    #[msg("Unauthorized admin")]
    Unauthorized,
    
    #[msg("Trading already paused")]
    AlreadyPaused,
    
    #[msg("Trading not paused")]
    NotPaused,
    
    #[msg("Invalid fee rate")]
    InvalidFeeRate,
    
    #[msg("Cannot withdraw while trading is active")]
    TradingActive,
}
