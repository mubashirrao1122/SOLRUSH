use anchor_lang::prelude::*;

#[error_code]
pub enum PoolError {
    #[msg("Invalid fee rate")]
    InvalidFeeRate,
    
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Invalid token amount")]
    InvalidAmount,
    
    #[msg("Pool reserves are zero")]
    ZeroReserves,
    
    #[msg("Arithmetic overflow")]
    Overflow,
    
    #[msg("Arithmetic underflow")]
    Underflow,
    
    #[msg("Invalid pool state")]
    InvalidPoolState,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Pool is paused")]
    PoolPaused,
    
    #[msg("LP token amount too small")]
    LPTokenAmountTooSmall,
    
    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Math calculation error")]
    MathError,
}
