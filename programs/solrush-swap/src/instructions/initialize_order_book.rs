use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(trading_pair: TradingPair)]
pub struct InitializeOrderBook<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"pool", trading_pair.seed()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        init,
        payer = authority,
        space = OrderBook::LEN,
        seeds = [b"order_book", trading_pair.seed()],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeOrderBook>,
    trading_pair: TradingPair,
) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;

    order_book.authority = ctx.accounts.authority.key();
    order_book.trading_pair = trading_pair;
    order_book.pool = ctx.accounts.pool.key();
    order_book.buy_orders_count = 0;
    order_book.sell_orders_count = 0;
    order_book.total_volume = 0;
    order_book.bump = ctx.bumps.order_book;

    msg!("Order book initialized for {:?}", trading_pair);

    Ok(())
}
