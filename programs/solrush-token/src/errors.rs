use anchor_lang::prelude::*;

#[error_code]
pub enum TokenError {
    #[msg("Supply cap exceeded")]
    SupplyCapExceeded,
    
    #[msg("Unauthorized mint authority")]
    UnauthorizedMintAuthority,
    
    #[msg("Invalid mint amount")]
    InvalidMintAmount,
    
    #[msg("Token already initialized")]
    AlreadyInitialized,
    
    #[msg("Arithmetic overflow")]
    Overflow,
    
    #[msg("Cannot mint more tokens, cap reached")]
    CapReached,
    
    #[msg("Invalid token state")]
    InvalidTokenState,
}
