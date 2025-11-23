use anchor_lang::prelude::*;

/// Trading pair enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TradingPair {
    SolUsdc,
    SolWeth,
    SolUsdt,
}

impl TradingPair {
    pub fn seed(&self) -> &[u8] {
        match self {
            TradingPair::SolUsdc => b"sol_usdc",
            TradingPair::SolWeth => b"sol_weth",
            TradingPair::SolUsdt => b"sol_usdt",
        }
    }
}

/// Order type enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    DCA,
}

/// Order side enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,  // Buy token (sell SOL)
    Sell, // Sell token (buy SOL)
}

/// Order status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
}

/// Liquidity pool configuration for AMM
#[account]
pub struct LiquidityPool {
    pub authority: Pubkey,
    pub trading_pair: TradingPair,
    pub token_a_mint: Pubkey,      // SOL (wrapped)
    pub token_b_mint: Pubkey,      // USDC/wETH/USDT
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub lp_token_mint: Pubkey,
    pub reserve_a: u64,            // SOL reserve
    pub reserve_b: u64,            // Token reserve
    pub lp_supply: u64,
    pub fee_rate: u16,             // Fee in basis points (30 = 0.3%)
    pub total_fees_a: u64,
    pub total_fees_b: u64,
    pub last_update_timestamp: i64,
    pub is_paused: bool,
    pub bump: u8,
}

impl LiquidityPool {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        1 +  // trading_pair
        32 + // token_a_mint
        32 + // token_b_mint
        32 + // token_a_vault
        32 + // token_b_vault
        32 + // lp_token_mint
        8 +  // reserve_a
        8 +  // reserve_b
        8 +  // lp_supply
        2 +  // fee_rate
        8 +  // total_fees_a
        8 +  // total_fees_b
        8 +  // last_update_timestamp
        1 +  // is_paused
        1;   // bump
}

/// Order book for limit orders
#[account]
pub struct OrderBook {
    pub authority: Pubkey,
    pub trading_pair: TradingPair,
    pub pool: Pubkey,
    pub buy_orders_count: u64,
    pub sell_orders_count: u64,
    pub total_volume: u64,
    pub bump: u8,
}

impl OrderBook {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        1 +  // trading_pair
        32 + // pool
        8 +  // buy_orders_count
        8 +  // sell_orders_count
        8 +  // total_volume
        1;   // bump
}

/// Limit order account
#[account]
pub struct LimitOrder {
    pub owner: Pubkey,
    pub order_book: Pubkey,
    pub trading_pair: TradingPair,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub amount_in: u64,
    pub amount_filled: u64,
    pub limit_price: u64,          // Price in token B per token A (scaled by 1e9)
    pub slippage_tolerance: u16,   // Basis points
    pub created_at: i64,
    pub expires_at: i64,           // 0 = never expires
    pub user_token_account: Pubkey,
    pub escrow_token_account: Pubkey,
    pub bump: u8,
}

impl LimitOrder {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // order_book
        1 +  // trading_pair
        1 +  // order_side
        1 +  // order_status
        8 +  // amount_in
        8 +  // amount_filled
        8 +  // limit_price
        2 +  // slippage_tolerance
        8 +  // created_at
        8 +  // expires_at
        32 + // user_token_account
        32 + // escrow_token_account
        1;   // bump
}

/// DCA (Dollar Cost Averaging) order account
#[account]
pub struct DCAOrder {
    pub owner: Pubkey,
    pub trading_pair: TradingPair,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub amount_per_cycle: u64,     // Amount to trade each cycle
    pub total_cycles: u16,         // Total number of cycles
    pub cycles_executed: u16,      // Cycles completed
    pub cycle_frequency: i64,      // Seconds between cycles
    pub last_execution: i64,       // Last execution timestamp
    pub next_execution: i64,       // Next scheduled execution
    pub slippage_tolerance: u16,   // Basis points
    pub min_price: u64,            // Min acceptable price (0 = no limit)
    pub max_price: u64,            // Max acceptable price (0 = no limit)
    pub user_token_account: Pubkey,
    pub escrow_token_account: Pubkey,
    pub total_amount_in: u64,      // Total deposited
    pub total_amount_out: u64,     // Total received
    pub bump: u8,
}

impl DCAOrder {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 +  // trading_pair
        1 +  // order_side
        1 +  // order_status
        8 +  // amount_per_cycle
        2 +  // total_cycles
        2 +  // cycles_executed
        8 +  // cycle_frequency
        8 +  // last_execution
        8 +  // next_execution
        2 +  // slippage_tolerance
        8 +  // min_price
        8 +  // max_price
        32 + // user_token_account
        32 + // escrow_token_account
        8 +  // total_amount_in
        8 +  // total_amount_out
        1;   // bump
}

/// Market order execution record
#[account]
pub struct MarketOrderRecord {
    pub owner: Pubkey,
    pub trading_pair: TradingPair,
    pub order_side: OrderSide,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_paid: u64,
    pub price_impact: u16,
    pub executed_at: i64,
    pub tx_signature: [u8; 64],
}

impl MarketOrderRecord {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 +  // trading_pair
        1 +  // order_side
        8 +  // amount_in
        8 +  // amount_out
        8 +  // fee_paid
        2 +  // price_impact
        8 +  // executed_at
        64;  // tx_signature
}

/// Swap quote information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub price_impact: u16, // in basis points
    pub minimum_received: u64,
}

/// Swap fee information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapFeeInfo {
    pub fee_rate: u16,
    pub total_fees_token_a: u64,
    pub total_fees_token_b: u64,
    pub lp_fee_share: u64, // percentage to LP providers
}

/// Swap route for multi-hop swaps (future enhancement)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapRoute {
    pub pools: Vec<Pubkey>,
    pub expected_output: u64,
    pub minimum_output: u64,
}
