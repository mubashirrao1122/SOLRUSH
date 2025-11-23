# SolRush DEX API Documentation

## Programs

1. solrush-liquidity-pool - AMM liquidity management
2. solrush-swap - Trading with market/limit/DCA orders
3. solrush-token - RUSH token (1M cap)
4. solrush-rewards - Liquidity mining
5. solrush-perpetual - Leveraged trading
6. solrush-admin - Platform controls

## Liquidity Pool

### initialize_pool
Create new liquidity pool.

**Parameters:**
- `fee_rate: u16` - Fee in basis points (30 = 0.3%)

**Accounts:**
- pool (PDA, init)
- token_a_mint, token_b_mint
- token_a_vault, token_b_vault (init)
- lp_token_mint (PDA, init)
- pool_authority (PDA)
- payer (signer)

### add_liquidity
Add liquidity and mint LP tokens.

**Parameters:**
- `token_a_amount: u64`
- `token_b_amount: u64`
- `min_lp_tokens: u64` - Slippage protection

**Returns:** LP tokens minted

### remove_liquidity
Burn LP tokens and withdraw liquidity.

**Parameters:**
- `lp_token_amount: u64`
- `min_token_a: u64`
- `min_token_b: u64`

### swap
Execute token swap.

**Parameters:**
- `amount_in: u64`
- `minimum_amount_out: u64`
- `input_is_token_a: bool`

### get_pool_info
Query pool state (reserves, LP supply, fee rate).

## Swap

### execute_market_order
Instant swap at market price.

**Parameters:**
- `amount_in: u64`
- `minimum_amount_out: u64`
- `slippage_tolerance: u16`
- `order_side: OrderSide` - Buy or Sell

### place_limit_order
Place order at target price.

**Parameters:**
- `trading_pair: TradingPair`
- `order_side: OrderSide`
- `amount_in: u64`
- `limit_price: u64` - Price scaled by 1e9
- `slippage_tolerance: u16`
- `expires_at: i64` - Unix timestamp (0 = never)

### execute_limit_order
Execute limit order when price reached (keeper).

### cancel_limit_order
Cancel order and return escrowed funds.

### create_dca_order
Setup recurring buy/sell order.

**Parameters:**
- `trading_pair: TradingPair`
- `order_side: OrderSide`
- `amount_per_cycle: u64`
- `total_cycles: u16`
- `cycle_frequency: i64` - Seconds between cycles
- `slippage_tolerance: u16`
- `min_price: u64` - Optional price floor
- `max_price: u64` - Optional price ceiling

### execute_dca_order
Execute next DCA cycle (keeper).

### cancel_dca_order
Cancel DCA and return remaining funds.

## Rush Token

### initialize_rush_token
Create RUSH token with 1M supply cap.

### mint_rush_tokens
Mint tokens (rewards contract only).

**Parameters:**
- `amount: u64`

**Access:** Restricted to rewards program

### get_total_supply
Query total minted supply.

### get_circulating_supply
Query circulating supply (total - locked).

## Rewards

### register_user
Register user for rewards tracking.

### claim_rewards
Claim accumulated rewards.

**Formula:** `(user_lp / total_lp) * time * reward_rate`

### update_reward_rate
Update reward rate (admin only).

**Parameters:**
- `pool_id: Pubkey`
- `new_rate: u64`

## Perpetual

### open_position
Open leveraged long/short position.

**Parameters:**
- `side: PositionSide` - Long or Short
- `size: u64`
- `leverage: u8` - 2x to 10x
- `collateral: u64`

### close_position
Close open position.

### add_margin
Add collateral to position.

**Parameters:**
- `additional_margin: u64`

### liquidate_position
Liquidate undercollateralized position.

## Admin

### pause_trading
Pause all trading (emergency).

### resume_trading
Resume trading after pause.

### update_fee_rate
Update pool fee rate (max 1%).

**Parameters:**
- `pool_id: Pubkey`
- `new_fee_rate: u16` - Max 100 (1%)

### transfer_authority
Transfer admin authority.

**Parameters:**
- `new_authority: Pubkey`

### emergency_withdraw
Withdraw funds when paused.

## Account Structures

### LiquidityPool
- token_a_reserve, token_b_reserve: u64
- lp_supply: u64
- fee_rate: u16
- authority: Pubkey
- is_paused: bool

### Position
- owner: Pubkey
- side: PositionSide
- size: u64
- leverage: u8
- entry_price: u64
- margin: u64
- last_funding_index: u64

### RewardInfo
- pool_id: Pubkey
- user: Pubkey
- lp_amount: u64
- reward_debt: u64
- last_claim_time: i64

## Errors

Common error codes:
- SlippageExceeded
- InsufficientLiquidity
- InvalidAmount
- InvalidFeeRate
- PoolPaused
- UnauthorizedAccess
- SupplyCapExceeded
- InsufficientMargin
- LiquidationThresholdReached
- InvalidLeverage
- OrderExpired
- LimitPriceNotReached

## Formulas

### AMM Swap
```
output = (reserve_out * amount_in * (10000 - fee)) / (reserve_in * 10000 + amount_in * (10000 - fee))
```

### LP Tokens
```
Initial: sqrt(amount_a * amount_b)
Subsequent: min(amount_a * total_lp / reserve_a, amount_b * total_lp / reserve_b)
```

### Liquidation Price
```
Long: entry_price * (1 - 1/leverage)
Short: entry_price * (1 + 1/leverage)
```

### PnL
```
Long: ((current_price - entry_price) / entry_price) * position_size
Short: ((entry_price - current_price) / entry_price) * position_size
```

## Events

Programs emit events for monitoring:
- PoolInitialized
- LiquidityAdded
- LiquidityRemoved
- SwapExecuted
- PositionOpened
- PositionClosed
- RewardsClaimed
- TradingPaused/Resumed
