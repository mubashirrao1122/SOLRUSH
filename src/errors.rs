use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Invalid vault account")]
    InvalidVault,
    #[msg("Invalid mint account")]
    InvalidMint,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("Insufficient liquidity in the pool")]
    InsufficientLiquidity,
    #[msg("Pool is not initialized")]
    PoolNotInitialized,
    #[msg("Invalid fee rate")]
    InvalidFeeRate,
    #[msg("Invalid amount")]
    InvalidAmount,
}
