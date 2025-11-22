use anchor_lang::prelude::*;

#[error_code]
pub enum RewardError {
    #[msg("Invalid reward rate")]
    InvalidRewardRate,
    
    #[msg("No rewards to claim")]
    NoRewardsToClaim,
    
    #[msg("Reward calculation overflow")]
    CalculationOverflow,
    
    #[msg("Supply cap would be exceeded")]
    SupplyCapExceeded,
    
    #[msg("User has no LP tokens")]
    NoLPTokens,
    
    #[msg("Invalid pool")]
    InvalidPool,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Math error")]
    MathError,
}
