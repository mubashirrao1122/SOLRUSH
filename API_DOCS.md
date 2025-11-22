# SolRush DEX API Documentation

## Table of Contents

1. [Liquidity Pool API](#liquidity-pool-api)
2. [Swap API](#swap-api)
3. [Rush Token API](#rush-token-api)
4. [Rewards API](#rewards-api)
5. [Perpetual Trading API](#perpetual-trading-api)
6. [Admin API](#admin-api)

## Liquidity Pool API

### Initialize Pool

Creates a new liquidity pool for a token pair.

**Instruction:** `initialize_pool`

**Parameters:**
- `fee_rate: u16` - Trading fee in basis points (e.g., 30 = 0.3%)

**Accounts:**
- `pool` - Pool state account (PDA, init)
- `token_a_mint` - Token A mint
- `token_b_mint` - Token B mint
- `token_a_vault` - Token A vault (init)
- `token_b_vault` - Token B vault (init)
- `lp_token_mint` - LP token mint (PDA, init)
- `pool_authority` - Pool authority (PDA)
- `payer` - Transaction fee payer (signer)

**Example:**
```typescript
await program.methods
  .initializePool(30) // 0.3% fee
  .accounts({ /* accounts */ })
  .rpc();
```


### Add Liquidity

Add liquidity to an existing pool and receive LP tokens.

**Instruction:** `add_liquidity`

**Parameters:**
- `token_a_amount: u64` - Amount of token A to deposit
- `token_b_amount: u64` - Amount of token B to deposit
- `min_lp_tokens: u64` - Minimum LP tokens to receive (slippage protection)

**Accounts:**
- `pool` - Pool state account
- `token_a_vault` - Token A vault
- `token_b_vault` - Token B vault
- `lp_token_mint` - LP token mint
- `user_token_a` - User's token A account
- `user_token_b` - User's token B account
- `user_lp_token` - User's LP token account
- `pool_authority` - Pool authority (PDA)
- `user` - User wallet (signer)

**Returns:** LP tokens minted to user

**Example:**
```typescript
await program.methods
  .addLiquidity(
    new BN(1000000),  // 1 token A
    new BN(1000000),  // 1 token B
    new BN(990000)    // min 0.99 LP tokens
  )
  .accounts({ /* accounts */ })
  .rpc();
```


### Remove Liquidity

Remove liquidity from a pool by burning LP tokens.

**Instruction:** `remove_liquidity`

**Parameters:**
- `lp_token_amount: u64` - Amount of LP tokens to burn
- `min_token_a: u64` - Minimum token A to receive
- `min_token_b: u64` - Minimum token B to receive

**Accounts:**
- `pool` - Pool state account
- `token_a_vault` - Token A vault
- `token_b_vault` - Token B vault
- `lp_token_mint` - LP token mint
- `user_token_a` - User's token A account
- `user_token_b` - User's token B account
- `user_lp_token` - User's LP token account
- `pool_authority` - Pool authority (PDA)
- `user` - User wallet (signer)

**Example:**
```typescript
await program.methods
  .removeLiquidity(
    new BN(500000),   // 0.5 LP tokens
    new BN(490000),   // min 0.49 token A
    new BN(490000)    // min 0.49 token B
  )
  .accounts({ /* accounts */ })
  .rpc();
```


### Swap

Execute a token swap.

**Instruction:** `swap`

**Parameters:**
- `amount_in: u64` - Input token amount
- `minimum_amount_out: u64` - Minimum output amount (slippage protection)

**Accounts:**
- `pool` - Pool state account
- `token_a_vault` - Token A vault
- `token_b_vault` - Token B vault
- `user_input_token` - User's input token account
- `user_output_token` - User's output token account
- `pool_authority` - Pool authority (PDA)
- `user` - User wallet (signer)

**Example:**
```typescript
await program.methods
  .swap(
    new BN(1000000),  // input amount
    new BN(990000)    // min output
  )
  .accounts({ /* accounts */ })
  .rpc();
```


## Swap API

### Execute Swap

Execute a swap with advanced validation.

**Instruction:** `execute_swap`

**Parameters:**
- `amount_in: u64` - Input amount
- `minimum_amount_out: u64` - Minimum output
- `slippage_tolerance: u16` - Slippage in basis points (max 1000 = 10%)

**Returns:** Swap completion event with price impact


### Calculate Swap Output

Get a quote for a swap without executing.

**Instruction:** `calculate_swap_output`

**Parameters:**
- `amount_in: u64` - Input amount
- `input_is_token_a: bool` - Whether input is token A

**Returns:**
```rust
pub struct SwapQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub price_impact: u16,
    pub minimum_received: u64,
}
```


## Rush Token API

### Initialize Rush Token

Initialize the RUSH token with supply cap.

**Instruction:** `initialize_rush_token`

**Accounts:**
- `token_state` - Token state account (PDA, init)
- `token_mint` - Token mint (init)
- `mint_authority` - Mint authority (PDA)
- `payer` - Transaction payer (signer)

**Token Specs:**
- Name: "Rush Token"
- Symbol: "RUSH"
- Decimals: 9
- Total Cap: 1,000,000 tokens


### Mint Rush Tokens

Mint new RUSH tokens (authority only).

**Instruction:** `mint_rush_tokens`

**Parameters:**
- `amount: u64` - Amount to mint

**Constraints:**
- Must not exceed supply cap
- Only authorized minter can call


## Rewards API

### Initialize Rewards

Initialize the reward distribution system.

**Instruction:** `initialize_rewards`

**Parameters:**
- `reward_rate_per_second: u64` - Reward emission rate


### Claim Rewards

Claim accumulated rewards.

**Instruction:** `claim_rewards`

**Returns:** RUSH tokens minted to user

**Formula:**
```
rewards = (user_lp_balance / total_lp_supply) * time_elapsed * reward_rate
```


## Perpetual Trading API

### Open Position

Open a leveraged position.

**Instruction:** `open_position`

**Parameters:**
- `side: PositionSide` - Long or Short
- `size: u64` - Position size
- `leverage: u8` - Leverage (2-10x)
- `collateral: u64` - Collateral amount

**Returns:** Position account

**Liquidation Prices:**
```
Long:  liq_price = entry_price * (1 - 1/leverage)
Short: liq_price = entry_price * (1 + 1/leverage)
```


### Close Position

Close an open position.

**Instruction:** `close_position`

**Constraints:**
- Only position owner can close
- Position must be open


### Liquidate Position

Liquidate an undercollateralized position (callable by anyone).

**Instruction:** `liquidate_position`

**Constraints:**
- Position must reach liquidation price
- Liquidator receives fee


## Admin API

### Pause Trading

Emergency pause all trading operations.

**Instruction:** `pause_trading`

**Parameters:**
- `reason: String` - Pause reason

**Constraints:**
- Only admin can call
- Trading must not already be paused


### Update Fee Rate

Update pool trading fee.

**Instruction:** `update_fee_rate`

**Parameters:**
- `new_rate: u16` - New fee rate (max 100 = 1%)

**Constraints:**
- Only admin can call
- Fee rate must be ≤ 1%


## Events

All instructions emit events for indexing:

### Pool Events
- `PoolInitialized` - Pool created
- `LiquidityAdded` - Liquidity added
- `LiquidityRemoved` - Liquidity removed
- `SwapExecuted` - Swap completed

### Token Events
- `RushTokenInitialized` - Token initialized
- `RushTokensMinted` - Tokens minted

### Reward Events
- `RewardsInitialized` - Rewards started
- `RewardsClaimed` - Rewards claimed

### Perpetual Events
- `PositionOpened` - Position opened
- `PositionClosed` - Position closed
- `PositionLiquidated` - Position liquidated

### Admin Events
- `TradingPaused` - Trading paused
- `TradingResumed` - Trading resumed
- `FeeRateUpdated` - Fee rate changed


## Error Codes

### Pool Errors
- `InvalidFeeRate` - Fee rate exceeds maximum
- `InsufficientLiquidity` - Not enough liquidity
- `SlippageExceeded` - Slippage tolerance exceeded
- `PoolPaused` - Operations paused

### Token Errors
- `SupplyCapExceeded` - Exceeds 1M cap
- `UnauthorizedMintAuthority` - Invalid minter

### Reward Errors
- `NoRewardsToClaim` - No pending rewards
- `CalculationOverflow` - Math overflow

### Perpetual Errors
- `InvalidLeverage` - Leverage out of range
- `Undercollateralized` - Insufficient margin
- `NotLiquidatable` - Position not liquidatable

### Admin Errors
- `Unauthorized` - Not admin
- `AlreadyPaused` - Already paused
- `InvalidFeeRate` - Invalid fee


## PDA Seeds

```rust
// Pool authority
["pool_authority", token_a_mint, token_b_mint]

// LP token mint
["lp_token", token_a_mint, token_b_mint]

// Pool state
["pool", token_a_mint, token_b_mint]

// Token state
["rush_token_state"]

// Mint authority
["mint_authority"]

// Reward state
["reward_state"]

// User rewards
["user_reward", user, pool]

// Position
["position", user, counter]

// Admin state
["admin_state"]
```


For more examples and integration guides, see the main README.md file.
