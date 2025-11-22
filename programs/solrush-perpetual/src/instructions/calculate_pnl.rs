use anchor_lang::prelude::*;
use crate::{state::{Position, PnLInfo}, utils::*};

#[derive(Accounts)]
pub struct CalculatePnL<'info> {
    pub position: Account<'info, Position>,
}

pub fn handler(ctx: Context<CalculatePnL>) -> Result<PnLInfo> {
    let position = &ctx.accounts.position;

    // Mock current price (use oracle in production)
    let current_price = 10500u64;

    let unrealized_pnl = calculate_unrealized_pnl(
        position.entry_price,
        current_price,
        position.size,
        position.side,
    )?;

    let is_liquidatable = should_liquidate(
        current_price,
        position.liquidation_price,
        position.side,
    );

    Ok(PnLInfo {
        unrealized_pnl,
        realized_pnl: 0,
        funding_payment: 0,
        liquidation_price: position.liquidation_price,
        is_liquidatable,
    })
}
