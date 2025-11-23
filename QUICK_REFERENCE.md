# SolRush DEX - Quick Reference

## Build & Deploy

```bash
# Build
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Test
anchor test

# Get program IDs
anchor keys list
```

## Common Operations

### Pool Operations
```typescript
// Initialize pool
await liquidityPool.methods.initializePool(30).accounts({...}).rpc();

// Add liquidity
await liquidityPool.methods
  .addLiquidity(new BN(1000000), new BN(1000000), new BN(990000))
  .accounts({...}).rpc();

// Remove liquidity
await liquidityPool.methods
  .removeLiquidity(new BN(1000000), new BN(0), new BN(0))
  .accounts({...}).rpc();
```

### Trading
```typescript
// Market order
await swap.methods
  .executeMarketOrder(
    new BN(1000000), // amount in
    new BN(990000),  // min out
    50,              // 0.5% slippage
    { buy: {} }
  )
  .accounts({...}).rpc();

// Limit order
await swap.methods
  .placeLimitOrder(
    { solUsdc: {} },
    { buy: {} },
    new BN(1000000),
    new BN(150000000000), // price
    100,
    0
  )
  .accounts({...}).rpc();

// DCA order
await swap.methods
  .createDcaOrder(
    { solUsdc: {} },
    { buy: {} },
    new BN(100000),  // per cycle
    30,              // cycles
    86400,           // frequency (seconds)
    200,
    new BN(0),
    new BN(0)
  )
  .accounts({...}).rpc();
```

### Rewards
```typescript
// Claim rewards
await rewards.methods.claimRewards().accounts({...}).rpc();
```

### Perpetuals
```typescript
// Open position
await perpetual.methods
  .openPosition({ long: {} }, new BN(10000000), 5, new BN(2000000))
  .accounts({...}).rpc();

// Close position
await perpetual.methods.closePosition().accounts({...}).rpc();
```

### Admin
```typescript
// Pause trading
await admin.methods.pauseTrading().accounts({...}).rpc();

// Update fee
await admin.methods.updateFeeRate(25).accounts({...}).rpc();
```

## PDA Derivations

```typescript
// Pool
const [pool] = PublicKey.findProgramAddressSync(
  [Buffer.from("pool"), tokenA.toBuffer(), tokenB.toBuffer()],
  programId
);

// Pool Authority
const [authority] = PublicKey.findProgramAddressSync(
  [Buffer.from("pool_authority"), tokenA.toBuffer(), tokenB.toBuffer()],
  programId
);

// LP Token
const [lpMint] = PublicKey.findProgramAddressSync(
  [Buffer.from("lp_token"), tokenA.toBuffer(), tokenB.toBuffer()],
  programId
);
```

## Important Constants

```rust
DEFAULT_FEE_RATE = 30        // 0.3%
MAX_FEE_RATE = 100          // 1%
MAX_SLIPPAGE = 1000         // 10%
RUSH_SUPPLY_CAP = 1_000_000 // 1M tokens
MIN_LEVERAGE = 2
MAX_LEVERAGE = 10
```

## Error Codes

Common errors:
- SlippageExceeded
- InsufficientLiquidity
- InvalidAmount
- PoolPaused
- UnauthorizedAccess
- SupplyCapExceeded
