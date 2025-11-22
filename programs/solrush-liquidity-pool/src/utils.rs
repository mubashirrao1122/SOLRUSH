use anchor_lang::prelude::*;
use crate::errors::PoolError;

/// Calculate LP tokens to mint for initial liquidity
pub fn calculate_initial_lp_tokens(token_a_amount: u64, token_b_amount: u64) -> Result<u64> {
    // Use geometric mean: sqrt(x * y)
    let product = (token_a_amount as u128)
        .checked_mul(token_b_amount as u128)
        .ok_or(PoolError::Overflow)?;
    
    let lp_tokens = sqrt_u128(product);
    
    require!(lp_tokens > 0, PoolError::InvalidAmount);
    
    Ok(lp_tokens as u64)
}

/// Calculate LP tokens to mint for additional liquidity
pub fn calculate_lp_tokens(
    token_a_amount: u64,
    token_b_amount: u64,
    token_a_reserve: u64,
    token_b_reserve: u64,
    lp_supply: u64,
) -> Result<u64> {
    require!(token_a_reserve > 0 && token_b_reserve > 0, PoolError::ZeroReserves);
    require!(lp_supply > 0, PoolError::InvalidPoolState);
    
    // Calculate based on minimum ratio to prevent manipulation
    let lp_from_a = (token_a_amount as u128)
        .checked_mul(lp_supply as u128)
        .ok_or(PoolError::Overflow)?
        .checked_div(token_a_reserve as u128)
        .ok_or(PoolError::MathError)?;
    
    let lp_from_b = (token_b_amount as u128)
        .checked_mul(lp_supply as u128)
        .ok_or(PoolError::Overflow)?
        .checked_div(token_b_reserve as u128)
        .ok_or(PoolError::MathError)?;
    
    let lp_tokens = std::cmp::min(lp_from_a, lp_from_b) as u64;
    
    require!(lp_tokens > 0, PoolError::LPTokenAmountTooSmall);
    
    Ok(lp_tokens)
}

/// Calculate tokens to return when removing liquidity
pub fn calculate_remove_liquidity(
    lp_token_amount: u64,
    token_a_reserve: u64,
    token_b_reserve: u64,
    lp_supply: u64,
) -> Result<(u64, u64)> {
    require!(lp_supply > 0, PoolError::InvalidPoolState);
    require!(lp_token_amount > 0, PoolError::InvalidAmount);
    
    let token_a_amount = (lp_token_amount as u128)
        .checked_mul(token_a_reserve as u128)
        .ok_or(PoolError::Overflow)?
        .checked_div(lp_supply as u128)
        .ok_or(PoolError::MathError)? as u64;
    
    let token_b_amount = (lp_token_amount as u128)
        .checked_mul(token_b_reserve as u128)
        .ok_or(PoolError::Overflow)?
        .checked_div(lp_supply as u128)
        .ok_or(PoolError::MathError)? as u64;
    
    require!(token_a_amount > 0 && token_b_amount > 0, PoolError::InsufficientLiquidity);
    
    Ok((token_a_amount, token_b_amount))
}

/// Calculate swap output using constant product formula with fees
/// Formula: output = (y * input * (10000 - fee)) / (x * 10000 + input * (10000 - fee))
pub fn calculate_swap_output(
    input_amount: u64,
    input_reserve: u64,
    output_reserve: u64,
    fee_rate: u16,
) -> Result<u64> {
    require!(input_amount > 0, PoolError::InvalidAmount);
    require!(input_reserve > 0 && output_reserve > 0, PoolError::ZeroReserves);
    
    // Calculate fee factor (10000 - fee_rate)
    let fee_factor = 10000u128
        .checked_sub(fee_rate as u128)
        .ok_or(PoolError::Underflow)?;
    
    // Numerator: output_reserve * input_amount * fee_factor
    let numerator = (output_reserve as u128)
        .checked_mul(input_amount as u128)
        .ok_or(PoolError::Overflow)?
        .checked_mul(fee_factor)
        .ok_or(PoolError::Overflow)?;
    
    // Denominator: input_reserve * 10000 + input_amount * fee_factor
    let denominator = (input_reserve as u128)
        .checked_mul(10000u128)
        .ok_or(PoolError::Overflow)?
        .checked_add(
            (input_amount as u128)
                .checked_mul(fee_factor)
                .ok_or(PoolError::Overflow)?
        )
        .ok_or(PoolError::Overflow)?;
    
    let output_amount = numerator
        .checked_div(denominator)
        .ok_or(PoolError::MathError)? as u64;
    
    require!(output_amount > 0, PoolError::InsufficientOutputAmount);
    require!(output_amount < output_reserve, PoolError::InsufficientLiquidity);
    
    Ok(output_amount)
}

/// Calculate fee amount from swap
pub fn calculate_fee_amount(amount: u64, fee_rate: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_rate as u128)
        .ok_or(PoolError::Overflow)?
        .checked_div(10000u128)
        .ok_or(PoolError::MathError)? as u64;
    
    Ok(fee)
}

/// Integer square root using Newton's method
fn sqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    
    let mut x = n;
    let mut y = (x + 1) / 2;
    
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    
    x
}

/// Validate slippage tolerance
pub fn validate_slippage(expected: u64, actual: u64, min_acceptable: u64) -> Result<()> {
    require!(actual >= min_acceptable, PoolError::SlippageExceeded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_u128() {
        assert_eq!(sqrt_u128(0), 0);
        assert_eq!(sqrt_u128(1), 1);
        assert_eq!(sqrt_u128(4), 2);
        assert_eq!(sqrt_u128(9), 3);
        assert_eq!(sqrt_u128(16), 4);
        assert_eq!(sqrt_u128(100), 10);
        assert_eq!(sqrt_u128(10000), 100);
    }

    #[test]
    fn test_calculate_swap_output() {
        // Test swap with 0.3% fee (30 basis points)
        let input = 1000;
        let input_reserve = 100000;
        let output_reserve = 50000;
        let fee_rate = 30;
        
        let result = calculate_swap_output(input, input_reserve, output_reserve, fee_rate);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output > 0);
        assert!(output < output_reserve);
    }

    #[test]
    fn test_calculate_initial_lp_tokens() {
        let result = calculate_initial_lp_tokens(1000000, 1000000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000000);
        
        let result2 = calculate_initial_lp_tokens(4000000, 1000000);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 2000000);
    }
}
