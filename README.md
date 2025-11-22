# SolRush DEX

## Overview

SolRush is a decentralized exchange on Solana with three liquidity pools (SOL/USDC, SOL/wETH, SOL/USDT), AMM-based trading, and a Rush token reward system with a 1,000,000 token supply cap.

## Architecture

The project consists of 6 interconnected Anchor programs:

1. **Liquidity Pool** - AMM-based liquidity pools with LP token minting
2. **Swap** - Token swap execution with slippage protection
3. **Rush Token** - Reward token with supply cap enforcement
4. **Rewards** - Liquidity mining and reward distribution
5. **Perpetual** - Leveraged trading with liquidation system
6. **Admin** - Platform administration and emergency controls

## Prerequisites

- Rust 1.70+
- Solana CLI 1.17+
- Anchor 0.29.0+
- Node.js 16+
- Yarn or npm

## Installation

### 1. Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.29.0
avm use 0.29.0

# Install Node dependencies
yarn install
```

### 2. Configure Solana

```bash
# Set to devnet
solana config set --url devnet

# Create wallet (if needed)
solana-keygen new --outfile ~/.config/solana/id.json

# Airdrop SOL for testing
solana airdrop 2
```

### 3. Build Programs

```bash
# Build all programs
anchor build

# Get program IDs
anchor keys list
```

### 4. Update Program IDs

Update the program IDs in `Anchor.toml` and `lib.rs` files with the output from `anchor keys list`.

## Deployment

### Deploy to Devnet

```bash
# Deploy all programs
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show <PROGRAM_ID>
```

### Initialize Programs

```bash
# Run initialization scripts
yarn run initialize-devnet
```

## Testing

```bash
# Run all tests
anchor test

# Run specific test file
anchor test --skip-deploy tests/liquidity-pool.ts

# Run with logs
anchor test -- --nocapture
```

## Program Details

### 1. Liquidity Pool (`solrush-liquidity-pool`)

**Instructions:**
- `initialize_pool(fee_rate)` - Create new liquidity pool
- `add_liquidity(token_a_amount, token_b_amount, min_lp_tokens)` - Add liquidity
- `remove_liquidity(lp_token_amount, min_token_a, min_token_b)` - Remove liquidity
- `swap(amount_in, minimum_amount_out)` - Execute token swap
- `get_pool_info()` - Get pool state

**Key Features:**
- Constant product AMM (x * y = k)
- 0.3% default trading fee
- LP token minting/burning
- Slippage protection

### 2. Swap (`solrush-swap`)

**Instructions:**
- `execute_swap(amount_in, minimum_amount_out, slippage_tolerance)` - Execute swap with validation
- `calculate_swap_output(amount_in, input_is_token_a)` - Get quote
- `get_swap_fee_info()` - Get fee information

**Key Features:**
- Price impact calculation
- Slippage validation (max 10%)
- Fee distribution to LPs

### 3. Rush Token (`solrush-token`)

**Instructions:**
- `initialize_rush_token()` - Initialize token mint
- `mint_rush_tokens(amount)` - Mint tokens (authority only)
- `transfer_mint_authority(new_authority)` - Transfer mint authority
- `get_total_supply()` - Get supply information
- `get_circulating_supply()` - Get circulating supply

**Token Details:**
- Name: Rush Token
- Symbol: RUSH
- Decimals: 9
- Total Supply Cap: 1,000,000 tokens (1,000,000,000,000,000 with decimals)

### 4. Rewards (`solrush-rewards`)

**Instructions:**
- `initialize_rewards(reward_rate_per_second)` - Initialize reward system
- `initialize_user_rewards()` - Create user reward account
- `update_user_rewards()` - Update accumulated rewards
- `claim_rewards()` - Claim pending rewards
- `calculate_pending_rewards()` - View pending rewards
- `update_reward_rate(new_rate)` - Update reward rate (admin)

**Reward Formula:**
```
rewards = (user_lp_balance / total_lp_supply) * time_elapsed * reward_rate
```

### 5. Perpetual (`solrush-perpetual`)

**Instructions:**
- `open_position(side, size, leverage, collateral)` - Open leveraged position
- `close_position()` - Close existing position
- `add_margin(additional_collateral)` - Add collateral to position
- `liquidate_position()` - Liquidate undercollateralized position
- `calculate_pnl()` - Calculate profit/loss
- `update_funding_rate(new_rate)` - Update funding rate

**Key Features:**
- Leverage: 2x-10x
- Long and short positions
- Automatic liquidation
- Funding rate mechanism

**Liquidation Formula:**
```
Long:  liquidation_price = entry_price * (1 - 1/leverage)
Short: liquidation_price = entry_price * (1 + 1/leverage)
```

### 6. Admin (`solrush-admin`)

**Instructions:**
- `initialize_admin()` - Initialize admin authority
- `pause_trading(reason)` - Emergency pause
- `resume_trading()` - Resume operations
- `update_fee_rate(new_rate)` - Update pool fees (max 1%)
- `transfer_admin(new_admin)` - Transfer admin authority
- `emergency_withdraw()` - Withdraw funds when paused

## Security Features

- Reentrancy guards on all value transfers
- Checked math operations (overflow/underflow protection)
- Signer validation for privileged operations
- PDA-based authority control
- Supply cap enforcement
- Slippage protection
- Emergency pause mechanism
- Liquidation safeguards

## API Documentation

### Example: Adding Liquidity

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolrushLiquidityPool } from "../target/types/solrush_liquidity_pool";

// Initialize provider
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.SolrushLiquidityPool as Program<SolrushLiquidityPool>;

// Add liquidity
await program.methods
  .addLiquidity(
    new anchor.BN(1000000), // token A amount
    new anchor.BN(1000000), // token B amount
    new anchor.BN(900000)   // min LP tokens
  )
  .accounts({
    pool: poolPda,
    tokenAVault: tokenAVaultPda,
    tokenBVault: tokenBVaultPda,
    lpTokenMint: lpMintPda,
    userTokenA: userTokenAAccount,
    userTokenB: userTokenBAccount,
    userLpToken: userLpAccount,
    poolAuthority: authorityPda,
    user: provider.wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### Example: Executing Swap

```typescript
await swapProgram.methods
  .executeSwap(
    new anchor.BN(1000),  // amount in
    new anchor.BN(990),   // minimum amount out
    50                    // slippage tolerance (0.5%)
  )
  .accounts({
    pool: poolPda,
    tokenAVault: tokenAVaultPda,
    tokenBVault: tokenBVaultPda,
    userInputToken: userInputAccount,
    userOutputToken: userOutputAccount,
    poolAuthority: authorityPda,
    user: provider.wallet.publicKey,
    liquidityPoolProgram: liquidityPoolProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### Example: Claiming Rewards

```typescript
await rewardsProgram.methods
  .claimRewards()
  .accounts({
    rewardState: rewardStatePda,
    userRewardAccount: userRewardPda,
    rushTokenMint: rushMintPda,
    userRushTokenAccount: userRushAccount,
    mintAuthority: mintAuthorityPda,
    user: provider.wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    rushTokenProgram: rushTokenProgram.programId,
  })
  .rpc();
```

## Transaction Fees

Estimated costs on Devnet/Mainnet:
- Initialize Pool: ~0.005 SOL
- Add Liquidity: ~0.002 SOL
- Swap: ~0.001 SOL
- Claim Rewards: ~0.001 SOL
- Open Position: ~0.003 SOL

## Development Workflow

### Local Testing

```bash
# Start local validator
solana-test-validator

# Deploy to localnet
anchor deploy --provider.cluster localnet

# Run tests
anchor test --skip-local-validator
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run security audit
cargo audit
```

## Project Structure

```
SOLRUSH/
├── Anchor.toml                 # Anchor configuration
├── Cargo.toml                  # Workspace configuration
├── README.md                   # This file
├── programs/
│   ├── solrush-liquidity-pool/ # AMM liquidity pool program
│   ├── solrush-swap/           # Swap execution program
│   ├── solrush-token/          # RUSH token program
│   ├── solrush-rewards/        # Reward distribution program
│   ├── solrush-perpetual/      # Perpetual trading program
│   └── solrush-admin/          # Administration program
├── tests/                      # Integration tests
├── scripts/                    # Deployment scripts
└── target/                     # Build output
```

## Troubleshooting

### Common Issues

**Program deployment fails:**
```bash
# Increase account balance
solana airdrop 5

# Check program size
ls -lh target/deploy/*.so
```

**Transaction simulation failed:**
- Check account ownership
- Verify PDA derivation
- Ensure sufficient token balances
- Check program IDs match

**Anchor build errors:**
```bash
# Clean and rebuild
anchor clean
anchor build
```

## License

MIT License - see LICENSE file for details

## Contributors

SolRush Development Team

**IMPORTANT:** This is development software. Audit thoroughly before mainnet deployment.
