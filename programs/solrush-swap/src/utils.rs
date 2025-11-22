use anchor_lang::prelude::*;
use crate::errors::SwapError;

/// Maximum slippage tolerance (10%)
pub const MAX_SLIPPAGE_TOLERANCE: u16 = 1000;

/// Maximum price impact allowed (5%)
pub const MAX_PRICE_IMPACT: u16 = 500;

/// Calculate swap output with fee
pub fn calculate_swap_with_fee(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_rate: u16,
) -> Result<(u64, u64)> {
    require!(amount_in > 0, SwapError::InvalidAmount);
    require!(reserve_in > 0 && reserve_out > 0, SwapError::InsufficientLiquidity);
    
    // Calculate fee
    let fee_amount = (amount_in as u128)
        .checked_mul(fee_rate as u128)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(10000u128)
        .ok_or(SwapError::MathError)? as u64;
    
    let amount_in_after_fee = amount_in
        .checked_sub(fee_amount)
        .ok_or(SwapError::MathError)?;
    
    // Calculate output using constant product formula
    // output = (reserve_out * amount_in_after_fee) / (reserve_in + amount_in_after_fee)
    let numerator = (reserve_out as u128)
        .checked_mul(amount_in_after_fee as u128)
        .ok_or(SwapError::CalculationOverflow)?;
    
    let denominator = (reserve_in as u128)
        .checked_add(amount_in_after_fee as u128)
        .ok_or(SwapError::CalculationOverflow)?;
    
    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(SwapError::MathError)? as u64;
    
    require!(amount_out > 0, SwapError::ZeroOutput);
    require!(amount_out < reserve_out, SwapError::InsufficientLiquidity);
    
    Ok((amount_out, fee_amount))
}

/// Calculate price impact in basis points
pub fn calculate_price_impact(
    amount_in: u64,
    amount_out: u64,
    reserve_in: u64,
    reserve_out: u64,
) -> Result<u16> {
    // Expected price without impact: reserve_out / reserve_in
    // Actual price: amount_out / amount_in
    // Price impact = (1 - actual_price / expected_price) * 10000
    
    let expected_out = (amount_in as u128)
        .checked_mul(reserve_out as u128)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(reserve_in as u128)
        .ok_or(SwapError::MathError)?;
    
    if expected_out == 0 {
        return Ok(0);
    }
    
    let impact = expected_out
        .checked_sub(amount_out as u128)
        .unwrap_or(0)
        .checked_mul(10000u128)
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(expected_out)
        .ok_or(SwapError::MathError)? as u16;
    
    Ok(impact)
}

/// Validate slippage tolerance
pub fn validate_slippage(
    expected_output: u64,
    minimum_output: u64,
    slippage_tolerance: u16,
) -> Result<()> {
    require!(
        slippage_tolerance <= MAX_SLIPPAGE_TOLERANCE,
        SwapError::InvalidSlippageTolerance
    );
    
    let min_acceptable = (expected_output as u128)
        .checked_mul((10000u128 - slippage_tolerance as u128))
        .ok_or(SwapError::CalculationOverflow)?
        .checked_div(10000u128)
        .ok_or(SwapError::MathError)? as u64;
    
    require!(
        minimum_output >= min_acceptable,
        SwapError::SlippageExceeded
    );
    
    Ok(())
}

/// Validate price impact
pub fn validate_price_impact(price_impact: u16) -> Result<()> {
    require!(
        price_impact <= MAX_PRICE_IMPACT,
        SwapError::PriceImpactTooHigh
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_swap_with_fee() {
        let amount_in = 1000u64;
        let reserve_in = 100000u64;
        let reserve_out = 50000u64;
        let fee_rate = 30u16; // 0.3%
        
        let result = calculate_swap_with_fee(amount_in, reserve_in, reserve_out, fee_rate);
        assert!(result.is_ok());
        
        let (amount_out, fee) = result.unwrap();
        assert!(amount_out > 0);
        assert!(fee > 0);
        assert_eq!(fee, 3); // 0.3% of 1000
    }

    #[test]
    fn test_calculate_price_impact() {
        let amount_in = 1000u64;
        let amount_out = 495u64;
        let reserve_in = 100000u64;
        let reserve_out = 50000u64;
        
        let result = calculate_price_impact(amount_in, amount_out, reserve_in, reserve_out);
        assert!(result.is_ok());
        
        let impact = result.unwrap();
        assert!(impact > 0);
        assert!(impact < 100); // Less than 1%
    }
}
