# SolRush Swap - Advanced Trading

Solana DEX swap program with market, limit, and DCA orders.

## Features

### Order Types
1. **Market Orders** - Instant execution at current price
2. **Limit Orders** - Execute when price target reached
3. **DCA Orders** - Recurring buys/sells with time intervals

### Trading Pairs
- SOL/USDC
- SOL/wETH
- SOL/USDT

### Core Features
- Constant product AMM (x * y = k)
- Slippage protection
- Price impact calculation
- 0.3% trading fee (configurable)
- Order book management
- Escrow system for pending orders

## Architecture

### Account Structures

**LiquidityPool** - AMM pool state
- Token reserves and vaults
- LP token supply
- Fee configuration

**OrderBook** - Limit order tracking
- Buy/sell order counts
- Volume tracking

**LimitOrder** - Individual orders
- Price, amount, side
- Escrow account
- Expiration timestamp

**DCAOrder** - Recurring orders
- Cycle configuration
- Price range limits
- Execution tracking

## Instructions

### Pool Management
- `initialize_pool` - Create liquidity pool
- `initialize_order_book` - Setup order book

### Market Orders
- `execute_market_order` - Instant swap with slippage protection

### Limit Orders
- `place_limit_order` - Place order in book
- `execute_limit_order` - Execute when price reached (keeper)
- `cancel_limit_order` - Cancel and return funds

### DCA Orders
- `create_dca_order` - Setup recurring order
- `execute_dca_order` - Execute next cycle (keeper)
- `cancel_dca_order` - Cancel and return remaining

## Formulas

### Swap Output
```
fee_adjusted = amount_in * (10000 - fee_rate) / 10000
output = (reserve_out * fee_adjusted) / (reserve_in * 10000 + fee_adjusted)
```

### Market Price
```
price = (reserve_b * 1e9) / reserve_a
```

### Price Impact
```
expected = reserve_out / reserve_in
actual = amount_out / amount_in
impact = ((expected - actual) / expected) * 10000
```

## Order Execution

### Limit Orders
1. User places order → tokens escrowed
2. Keeper monitors prices
3. When price target met → execute swap
4. Send output to order owner

### DCA Orders
1. User creates order → full amount escrowed
2. Keeper monitors time
3. When next_execution reached → execute one cycle
4. Repeat until all cycles complete

## PDA Seeds

```rust
Pool: ["pool", trading_pair.seed()]
OrderBook: ["order_book", trading_pair.seed()]
LimitOrder: ["limit_order", user, order_book, index]
DCAOrder: ["dca_order", user, timestamp]
```

## Fee Structure

Trading fee: 0.3% (30 basis points)
Distribution: 100% to liquidity providers

## Security

- Checked arithmetic (overflow/underflow protection)
- Slippage guards
- Price impact limits
- Ownership validation
- Escrow security via PDAs
- Time-based execution guards

## Known Issues

Minor PDA seed compilation errors. See QUICK_FIX_GUIDE.md for fixes.

## Client Example

See `client-example.ts` for TypeScript SDK usage examples.
