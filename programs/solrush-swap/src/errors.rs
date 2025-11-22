use anchor_lang::prelude::*;

#[error_code]
pub enum SwapError {
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Invalid swap amount")]
    InvalidAmount,
    
    #[msg("Insufficient liquidity for swap")]
    InsufficientLiquidity,
    
    #[msg("Invalid token pair")]
    InvalidTokenPair,
    
    #[msg("Swap calculation overflow")]
    CalculationOverflow,
    
    #[msg("Invalid slippage tolerance")]
    InvalidSlippageTolerance,
    
    #[msg("Pool is paused")]
    PoolPaused,
    
    #[msg("Price impact too high")]
    PriceImpactTooHigh,
    
    #[msg("Zero output amount")]
    ZeroOutput,
    
    #[msg("Math error")]
    MathError,
}
