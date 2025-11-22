use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{constants::*, errors::PerpetualError, state::{Position, PositionSide}, utils::*};

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(
        init,
        payer = user,
        space = Position::LEN,
        seeds = [
            POSITION_SEED,
            user.key().as_ref(),
            &[position_counter],
        ],
        bump
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: Trading pair reference
    pub pair: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<OpenPosition>,
    side: PositionSide,
    size: u64,
    leverage: u8,
    collateral: u64,
) -> Result<()> {
    require!(size > 0, PerpetualError::InvalidPositionSize);
    require!(leverage >= MIN_LEVERAGE && leverage <= MAX_LEVERAGE, PerpetualError::InvalidLeverage);

    let required_margin = calculate_required_margin(size, leverage)?;
    require!(collateral >= required_margin, PerpetualError::InsufficientCollateral);

    // Mock entry price (in production, use oracle)
    let entry_price = 10000u64;

    let liquidation_price = calculate_liquidation_price(entry_price, leverage, side)?;

    // Transfer collateral to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_collateral_account.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        collateral,
    )?;

    let position = &mut ctx.accounts.position;
    let clock = Clock::get()?;

    position.bump = ctx.bumps.position;
    position.owner = ctx.accounts.user.key();
    position.pair = ctx.accounts.pair.key();
    position.side = side;
    position.size = size;
    position.leverage = leverage;
    position.collateral = collateral;
    position.entry_price = entry_price;
    position.liquidation_price = liquidation_price;
    position.take_profit = None;
    position.stop_loss = None;
    position.funding_index = 0;
    position.is_open = true;
    position.opened_at = clock.unix_timestamp;
    position.closed_at = None;

    emit!(PositionOpened {
        position: position.key(),
        owner: position.owner,
        side,
        size,
        leverage,
        collateral,
        entry_price,
        liquidation_price,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PositionOpened {
    pub position: Pubkey,
    pub owner: Pubkey,
    pub side: PositionSide,
    pub size: u64,
    pub leverage: u8,
    pub collateral: u64,
    pub entry_price: u64,
    pub liquidation_price: u64,
    pub timestamp: i64,
}
