use anchor_lang::prelude::*;
use crate::{errors::PerpetualError, state::{Position, PnLInfo}, utils::*};

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    #[account(
        mut,
        constraint = position.is_open @ PerpetualError::PositionClosed,
    )]
    pub position: Account<'info, Position>,

    pub liquidator: Signer<'info>,
}

pub fn handler(ctx: Context<LiquidatePosition>) -> Result<()> {
    let position = &mut ctx.accounts.position;

    // Mock current price (use oracle in production)
    let current_price = 9500u64;

    let should_liq = should_liquidate(
        current_price,
        position.liquidation_price,
        position.side,
    );

    require!(should_liq, PerpetualError::NotLiquidatable);

    position.is_open = false;
    position.closed_at = Some(Clock::get()?.unix_timestamp);

    emit!(PositionLiquidated {
        position: position.key(),
        owner: position.owner,
        liquidator: ctx.accounts.liquidator.key(),
        liquidation_price: position.liquidation_price,
        current_price,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PositionLiquidated {
    pub position: Pubkey,
    pub owner: Pubkey,
    pub liquidator: Pubkey,
    pub liquidation_price: u64,
    pub current_price: u64,
    pub timestamp: i64,
}
