use anchor_lang::prelude::*;
use crate::{constants::*, errors::PerpetualError, state::PositionSide};

/// Calculate required margin for a position
pub fn calculate_required_margin(size: u64, leverage: u8) -> Result<u64> {
    require!(leverage >= MIN_LEVERAGE && leverage <= MAX_LEVERAGE, PerpetualError::InvalidLeverage);
    
    let margin = (size as u128)
        .checked_div(leverage as u128)
        .ok_or(PerpetualError::Overflow)? as u64;
    
    Ok(margin)
}

/// Calculate liquidation price
pub fn calculate_liquidation_price(
    entry_price: u64,
    leverage: u8,
    side: PositionSide,
) -> Result<u64> {
    let leverage_ratio = leverage as u128;
    
    let liq_price = match side {
        PositionSide::Long => {
            // Long: liq_price = entry_price * (1 - 1/leverage)
            let factor = (leverage_ratio - 1) * 10000 / leverage_ratio;
            (entry_price as u128)
                .checked_mul(factor)
                .ok_or(PerpetualError::Overflow)?
                .checked_div(10000)
                .ok_or(PerpetualError::Overflow)? as u64
        }
        PositionSide::Short => {
            // Short: liq_price = entry_price * (1 + 1/leverage)
            let factor = (leverage_ratio + 1) * 10000 / leverage_ratio;
            (entry_price as u128)
                .checked_mul(factor)
                .ok_or(PerpetualError::Overflow)?
                .checked_div(10000)
                .ok_or(PerpetualError::Overflow)? as u64
        }
    };
    
    Ok(liq_price)
}

/// Calculate unrealized PnL
pub fn calculate_unrealized_pnl(
    entry_price: u64,
    current_price: u64,
    size: u64,
    side: PositionSide,
) -> Result<i64> {
    let pnl = match side {
        PositionSide::Long => {
            // Long: PnL = size * (current_price - entry_price) / entry_price
            let price_diff = (current_price as i128) - (entry_price as i128);
            let pnl = (size as i128)
                .checked_mul(price_diff)
                .ok_or(PerpetualError::Overflow)?
                .checked_div(entry_price as i128)
                .ok_or(PerpetualError::Overflow)?;
            pnl as i64
        }
        PositionSide::Short => {
            // Short: PnL = size * (entry_price - current_price) / entry_price
            let price_diff = (entry_price as i128) - (current_price as i128);
            let pnl = (size as i128)
                .checked_mul(price_diff)
                .ok_or(PerpetualError::Overflow)?
                .checked_div(entry_price as i128)
                .ok_or(PerpetualError::Overflow)?;
            pnl as i64
        }
    };
    
    Ok(pnl)
}

/// Check if position should be liquidated
pub fn should_liquidate(
    current_price: u64,
    liquidation_price: u64,
    side: PositionSide,
) -> bool {
    match side {
        PositionSide::Long => current_price <= liquidation_price,
        PositionSide::Short => current_price >= liquidation_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_required_margin() {
        let size = 10000u64;
        let leverage = 5u8;
        let result = calculate_required_margin(size, leverage);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2000);
    }

    #[test]
    fn test_calculate_liquidation_price() {
        let entry_price = 10000u64;
        let leverage = 5u8;
        
        let long_liq = calculate_liquidation_price(entry_price, leverage, PositionSide::Long);
        assert!(long_liq.is_ok());
        assert!(long_liq.unwrap() < entry_price);
        
        let short_liq = calculate_liquidation_price(entry_price, leverage, PositionSide::Short);
        assert!(short_liq.is_ok());
        assert!(short_liq.unwrap() > entry_price);
    }
}
