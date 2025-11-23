use anchor_lang::prelude::*;

declare_id!("SRSwap1111111111111111111111111111111111111");

pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod solrush_swap {
    use super::*;

    /// Initialize a liquidity pool for a trading pair
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        trading_pair: TradingPair,
        fee_rate: u16,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, trading_pair, fee_rate)
    }

    /// Initialize an order book for limit orders
    pub fn initialize_order_book(
        ctx: Context<InitializeOrderBook>,
        trading_pair: TradingPair,
    ) -> Result<()> {
        instructions::initialize_order_book::handler(ctx, trading_pair)
    }

    /// Execute a market order with instant AMM execution
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `amount_in` - Input token amount
    /// * `minimum_amount_out` - Minimum output amount (slippage protection)
    /// * `slippage_tolerance` - Slippage tolerance in basis points
    /// * `order_side` - Buy or Sell
    pub fn execute_market_order(
        ctx: Context<ExecuteMarketOrder>,
        amount_in: u64,
        minimum_amount_out: u64,
        slippage_tolerance: u16,
        order_side: OrderSide,
    ) -> Result<()> {
        instructions::execute_market_order::handler(
            ctx,
            amount_in,
            minimum_amount_out,
            slippage_tolerance,
            order_side,
        )
    }

    /// Place a limit order
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `trading_pair` - Trading pair enum
    /// * `order_side` - Buy or Sell
    /// * `amount_in` - Input token amount
    /// * `limit_price` - Limit price (scaled by 1e9)
    /// * `slippage_tolerance` - Slippage tolerance in basis points
    /// * `expires_at` - Expiration timestamp (0 = never expires)
    pub fn place_limit_order(
        ctx: Context<PlaceLimitOrder>,
        trading_pair: TradingPair,
        order_side: OrderSide,
        amount_in: u64,
        limit_price: u64,
        slippage_tolerance: u16,
        expires_at: i64,
    ) -> Result<()> {
        instructions::place_limit_order::handler(
            ctx,
            trading_pair,
            order_side,
            amount_in,
            limit_price,
            slippage_tolerance,
            expires_at,
        )
    }

    /// Cancel a limit order
    pub fn cancel_limit_order(ctx: Context<CancelLimitOrder>) -> Result<()> {
        instructions::cancel_limit_order::handler(ctx)
    }

    /// Execute a limit order when price conditions are met
    pub fn execute_limit_order(ctx: Context<ExecuteLimitOrder>) -> Result<()> {
        instructions::execute_limit_order::handler(ctx)
    }

    /// Create a DCA (Dollar Cost Averaging) order
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `trading_pair` - Trading pair enum
    /// * `order_side` - Buy or Sell
    /// * `amount_per_cycle` - Amount to trade each cycle
    /// * `total_cycles` - Total number of cycles
    /// * `cycle_frequency` - Seconds between cycles
    /// * `slippage_tolerance` - Slippage tolerance in basis points
    /// * `min_price` - Minimum acceptable price (0 = no limit)
    /// * `max_price` - Maximum acceptable price (0 = no limit)
    pub fn create_dca_order(
        ctx: Context<CreateDCAOrder>,
        trading_pair: TradingPair,
        order_side: OrderSide,
        amount_per_cycle: u64,
        total_cycles: u16,
        cycle_frequency: i64,
        slippage_tolerance: u16,
        min_price: u64,
        max_price: u64,
    ) -> Result<()> {
        instructions::create_dca_order::handler(
            ctx,
            trading_pair,
            order_side,
            amount_per_cycle,
            total_cycles,
            cycle_frequency,
            slippage_tolerance,
            min_price,
            max_price,
        )
    }

    /// Execute a DCA order cycle
    pub fn execute_dca_order(ctx: Context<ExecuteDCAOrder>) -> Result<()> {
        instructions::execute_dca_order::handler(ctx)
    }

    /// Cancel a DCA order and return remaining funds
    pub fn cancel_dca_order(ctx: Context<CancelDCAOrder>) -> Result<()> {
        instructions::cancel_dca_order::handler(ctx)
    }

    /// Execute a token swap with slippage protection (legacy)
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `amount_in` - Input token amount
    /// * `minimum_amount_out` - Minimum output amount (slippage protection)
    /// * `slippage_tolerance` - Slippage tolerance in basis points
    pub fn execute_swap(
        ctx: Context<ExecuteSwap>,
        amount_in: u64,
        minimum_amount_out: u64,
        slippage_tolerance: u16,
    ) -> Result<()> {
        instructions::execute_swap::handler(ctx, amount_in, minimum_amount_out, slippage_tolerance)
    }

    /// Calculate expected swap output
    /// 
    /// # Arguments
    /// * `ctx` - Context with pool account
    /// * `amount_in` - Input token amount
    /// * `input_is_token_a` - Whether input is token A
    pub fn calculate_swap_output(
        ctx: Context<CalculateSwapOutput>,
        amount_in: u64,
        input_is_token_a: bool,
    ) -> Result<SwapQuote> {
        instructions::calculate_swap_output::handler(ctx, amount_in, input_is_token_a)
    }

    /// Get swap fee information
    pub fn get_swap_fee_info(ctx: Context<GetSwapFeeInfo>) -> Result<SwapFeeInfo> {
        instructions::get_swap_fee_info::handler(ctx)
    }
}
