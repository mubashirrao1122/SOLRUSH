use anchor_lang::prelude::*;

declare_id!("SRPerp1111111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod solrush_perpetual {
    use super::*;

    /// Open a leveraged position
    pub fn open_position(
        ctx: Context<OpenPosition>,
        side: PositionSide,
        size: u64,
        leverage: u8,
        collateral: u64,
    ) -> Result<()> {
        instructions::open_position::handler(ctx, side, size, leverage, collateral)
    }

    /// Close an existing position
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::close_position::handler(ctx)
    }

    /// Add margin to an existing position
    pub fn add_margin(ctx: Context<AddMargin>, additional_collateral: u64) -> Result<()> {
        instructions::add_margin::handler(ctx, additional_collateral)
    }

    /// Liquidate an undercollateralized position
    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        instructions::liquidate_position::handler(ctx)
    }

    /// Calculate PnL for a position
    pub fn calculate_pnl(ctx: Context<CalculatePnL>) -> Result<PnLInfo> {
        instructions::calculate_pnl::handler(ctx)
    }

    /// Update funding rate
    pub fn update_funding_rate(ctx: Context<UpdateFundingRate>, new_rate: i64) -> Result<()> {
        instructions::update_funding_rate::handler(ctx, new_rate)
    }
}
