use anchor_lang::prelude::*;
use crate::state::Pool;
use crate::events::FeesUpdated;
use crate::errors::AmmError;

#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
        constraint = pool.admin == admin.key() @ AmmError::Unauthorized,
    )]
    pub pool: Account<'info, Pool>,
}

pub fn handler(
    ctx: Context<UpdateFees>,
    fee_rate_bps: u16,
    protocol_fee_rate_bps: u16,
) -> Result<()> {
    require!(fee_rate_bps <= 10000, AmmError::InvalidFeeRate);
    require!(protocol_fee_rate_bps <= 10000, AmmError::InvalidFeeRate);

    let pool = &mut ctx.accounts.pool;
    pool.fee_rate_bps = fee_rate_bps;
    pool.protocol_fee_rate_bps = protocol_fee_rate_bps;

    emit!(FeesUpdated {
        pool: pool.key(),
        new_fee_rate_bps: fee_rate_bps,
        new_protocol_fee_rate_bps: protocol_fee_rate_bps,
    });

    Ok(())
}
