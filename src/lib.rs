use anchor_lang::prelude::*;

pub mod state;
pub mod errors;
pub mod events;
pub mod instructions;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"); // Default Anchor ID, user should replace

#[program]
pub mod dex_amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee_rate_bps: u16,
        protocol_fee_rate_bps: u16,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, fee_rate_bps, protocol_fee_rate_bps)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        instructions::add_liquidity::handler(ctx, amount_a, amount_b)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, lp_amount)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, min_amount_out)
    }

    pub fn update_fees(
        ctx: Context<UpdateFees>,
        fee_rate_bps: u16,
        protocol_fee_rate_bps: u16,
    ) -> Result<()> {
        instructions::update_fees::handler(ctx, fee_rate_bps, protocol_fee_rate_bps)
    }
}
