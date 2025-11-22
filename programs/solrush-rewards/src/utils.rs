use anchor_lang::prelude::*;
use crate::errors::RewardError;

/// Calculate rewards based on LP share and time
pub fn calculate_rewards(
    lp_balance: u64,
    total_lp_supply: u64,
    time_elapsed: i64,
    reward_rate: u64,
) -> Result<u64> {
    require!(total_lp_supply > 0, RewardError::InvalidPool);
    require!(time_elapsed >= 0, RewardError::MathError);
    
    if lp_balance == 0 || time_elapsed == 0 {
        return Ok(0);
    }
    
    // rewards = (lp_balance / total_lp) * time_elapsed * reward_rate
    let lp_share = (lp_balance as u128)
        .checked_mul(1_000_000_000u128)
        .ok_or(RewardError::CalculationOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(RewardError::MathError)?;
    
    let time_rewards = lp_share
        .checked_mul(time_elapsed as u128)
        .ok_or(RewardError::CalculationOverflow)?
        .checked_mul(reward_rate as u128)
        .ok_or(RewardError::CalculationOverflow)?
        .checked_div(1_000_000_000u128)
        .ok_or(RewardError::MathError)? as u64;
    
    Ok(time_rewards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rewards() {
        let lp_balance = 1000u64;
        let total_lp = 10000u64;
        let time_elapsed = 86400i64; // 1 day
        let reward_rate = 1000u64;
        
        let result = calculate_rewards(lp_balance, total_lp, time_elapsed, reward_rate);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
