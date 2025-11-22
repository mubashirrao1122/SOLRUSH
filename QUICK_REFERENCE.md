# SolRush DEX - Quick Reference

## Quick Commands

```bash
# Build all programs
anchor build

# Deploy to Devnet
anchor deploy --provider.cluster devnet

# Run tests
anchor test

# Deploy with script
bash scripts/deploy.sh

# Initialize programs
bash scripts/initialize.sh
```

## Common Operations

### Create a Pool

```typescript
await liquidityPoolProgram.methods
  .initializePool(30) // 0.3% fee
  .accounts({...})
  .rpc();
```

### Add Liquidity

```typescript
await liquidityPoolProgram.methods
  .addLiquidity(
    new BN(1_000_000_000), // 1 token A
    new BN(1_000_000_000), // 1 token B
    new BN(990_000_000)    // min LP tokens
  )
  .accounts({...})
  .rpc();
```

### Swap Tokens

```typescript
await swapProgram.methods
  .executeSwap(
    new BN(1_000_000),    // amount in
    new BN(990_000),      // min out
    50                    // 0.5% slippage
  )
  .accounts({...})
  .rpc();
```

### Claim Rewards

```typescript
await rewardsProgram.methods
  .claimRewards()
  .accounts({...})
  .rpc();
```

### Open Position

```typescript
await perpetualProgram.methods
  .openPosition(
    { long: {} },        // side
    new BN(10_000_000),  // size
    5,                   // 5x leverage
    new BN(2_000_000)    // collateral
  )
  .accounts({...})
  .rpc();
```

## PDA Derivations

```typescript
// Pool PDA
const [poolPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("pool"), tokenAMint.toBuffer(), tokenBMint.toBuffer()],
  programId
);

// Pool Authority
const [authorityPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("pool_authority"), tokenAMint.toBuffer(), tokenBMint.toBuffer()],
  programId
);

// LP Token Mint
const [lpMintPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("lp_token"), tokenAMint.toBuffer(), tokenBMint.toBuffer()],
  programId
);

// Token State
const [tokenStatePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("rush_token_state")],
  programId
);

// Reward State
const [rewardStatePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("reward_state")],
  programId
);

// User Reward Account
const [userRewardPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("user_reward"), user.toBuffer(), pool.toBuffer()],
  programId
);

// Admin State
const [adminStatePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("admin_state")],
  programId
);
```

## Program IDs (Update after deployment)

```typescript
const LIQUIDITY_POOL_PROGRAM_ID = new PublicKey("SRLPooL111...");
const SWAP_PROGRAM_ID = new PublicKey("SRSwap1111...");
const TOKEN_PROGRAM_ID = new PublicKey("SRToken111...");
const REWARDS_PROGRAM_ID = new PublicKey("SRReward11...");
const PERPETUAL_PROGRAM_ID = new PublicKey("SRPerp1111...");
const ADMIN_PROGRAM_ID = new PublicKey("SRAdmin111...");
```

## Token Amounts

```typescript
// With 9 decimals
const ONE_TOKEN = new BN(1_000_000_000);
const HALF_TOKEN = new BN(500_000_000);
const THOUSAND_TOKENS = new BN(1_000_000_000_000);
const ONE_MILLION = new BN(1_000_000_000_000_000); // RUSH cap
```

## Configuration

### Fee Rates (basis points)
- Default: 30 (0.3%)
- Maximum: 100 (1%)

### Leverage
- Minimum: 2x
- Maximum: 10x

### Slippage
- Maximum: 1000 basis points (10%)

### Token Supply
- Rush Token Cap: 1,000,000 tokens

## Fetching Account Data

```typescript
// Get pool state
const poolAccount = await program.account.poolState.fetch(poolPda);
console.log("Token A Reserve:", poolAccount.tokenAReserve.toString());
console.log("Token B Reserve:", poolAccount.tokenBReserve.toString());
console.log("LP Supply:", poolAccount.lpTokenSupply.toString());

// Get token state
const tokenState = await tokenProgram.account.tokenState.fetch(tokenStatePda);
console.log("Total Minted:", tokenState.totalMinted.toString());
console.log("Remaining:", tokenState.supplyCapTokenState.totalMinted.toString());

// Get user rewards
const userReward = await rewardsProgram.account.userRewardAccount.fetch(userRewardPda);
console.log("Earned:", userReward.earnedRewards.toString());
console.log("Claimed:", userReward.claimedRewards.toString());
```

## Troubleshooting

### "Transaction simulation failed"
- Check all account addresses are correct
- Verify PDAs are derived correctly
- Ensure sufficient token balances
- Check signer authorities

### "Custom program error: 0x..."
- Look up error code in program errors.rs
- Common: 0x1770 = Insufficient liquidity
- Common: 0x1771 = Slippage exceeded

### "Program failed to complete"
- Check compute unit limits
- May need to increase transaction size
- Split large operations

### Build errors
```bash
anchor clean
rm -rf target/
anchor build
```

## Best Practices

1. **Always use slippage protection**
   ```typescript
   const minOut = expectedOut * 0.99; // 1% slippage
   ```

2. **Check pool reserves before swaps**
   ```typescript
   const pool = await program.account.poolState.fetch(poolPda);
   if (pool.tokenAReserve < minRequired) {
     throw new Error("Insufficient liquidity");
   }
   ```

3. **Use transaction confirmation**
   ```typescript
   const tx = await program.methods.swap(...).rpc();
   await provider.connection.confirmTransaction(tx, "confirmed");
   ```

4. **Handle errors gracefully**
   ```typescript
   try {
     await program.methods.claimRewards().rpc();
   } catch (e) {
     if (e.message.includes("NoRewardsToClaim")) {
       console.log("No rewards available");
     }
   }
   ```

## Security Checklist

- [ ] Always validate token mint addresses
- [ ] Check pool is not paused before operations
- [ ] Verify sufficient balances before transactions
- [ ] Use minimum output parameters
- [ ] Monitor position health for liquidations
- [ ] Keep admin keys secure (use hardware wallet)
- [ ] Test on devnet thoroughly first

## Monitoring

### Important Events to Listen For

```typescript
// Pool events
program.addEventListener("LiquidityAdded", (event) => {
  console.log("Liquidity added:", event);
});

program.addEventListener("SwapExecuted", (event) => {
  console.log("Swap executed:", event);
});

// Reward events
rewardsProgram.addEventListener("RewardsClaimed", (event) => {
  console.log("Rewards claimed:", event);
});

// Admin events
adminProgram.addEventListener("TradingPaused", (event) => {
  console.warn("TRADING PAUSED:", event.reason);
});
```

## Useful Calculations

### Calculate LP Tokens (Initial)
```typescript
const lpTokens = Math.sqrt(tokenAAmount * tokenBAmount);
```

### Calculate Swap Output
```typescript
const fee = 0.003; // 0.3%
const amountAfterFee = amountIn * (1 - fee);
const output = (reserveOut * amountAfterFee) / (reserveIn + amountAfterFee);
```

### Calculate Price Impact
```typescript
const expectedPrice = reserveOut / reserveIn;
const actualPrice = amountOut / amountIn;
const impact = (1 - actualPrice / expectedPrice) * 100;
```

### Calculate Required Margin
```typescript
const requiredMargin = positionSize / leverage;
```

## Additional Resources

- Full Documentation: `README.md`
- API Reference: `API_DOCS.md`
- Project Summary: `SUMMARY.md`
- Example Tests: `tests/liquidity-pool.ts`

## Tips

1. Use Anchor's automatic PDA derivation in IDL
2. Batch multiple operations when possible
3. Monitor on-chain events for real-time updates
4. Keep track of reward accrual times
5. Set appropriate leverage based on volatility
6. Always maintain sufficient collateral
7. Use view functions to preview operations
