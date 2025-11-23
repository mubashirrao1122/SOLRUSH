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
    
    // Order-related errors
    #[msg("Order not found")]
    OrderNotFound,
    
    #[msg("Order already filled")]
    OrderAlreadyFilled,
    
    #[msg("Order expired")]
    OrderExpired,
    
    #[msg("Order already cancelled")]
    OrderAlreadyCancelled,
    
    #[msg("Unauthorized order access")]
    UnauthorizedOrderAccess,
    
    #[msg("Invalid limit price")]
    InvalidLimitPrice,
    
    #[msg("Limit price not reached")]
    LimitPriceNotReached,
    
    #[msg("Invalid DCA parameters")]
    InvalidDCAParameters,
    
    #[msg("DCA order not ready for execution")]
    DCANotReady,
    
    #[msg("DCA order completed")]
    DCACompleted,
    
    #[msg("Insufficient escrow balance")]
    InsufficientEscrowBalance,
    
    #[msg("Invalid order type")]
    InvalidOrderType,
    
    #[msg("Invalid order status")]
    InvalidOrderStatus,
    
    #[msg("Price out of acceptable range")]
    PriceOutOfRange,
    
    #[msg("Order book full")]
    OrderBookFull,
    
    #[msg("Invalid cycle frequency")]
    InvalidCycleFrequency,
    
    #[msg("Maximum cycles exceeded")]
    MaxCyclesExceeded,
    
    #[msg("Invalid expiration time")]
    InvalidExpirationTime,
}
