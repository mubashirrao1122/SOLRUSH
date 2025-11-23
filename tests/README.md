# SolRush DEX Test Suite

## Structure

```
tests/
├── utils/test-helpers.ts              # Shared utilities
├── unit/amm_calculations.test.ts      # Math formulas (29 tests)
└── integration/pool_operations.test.ts # Pool workflows
```

## Running Tests

```bash
# All tests
anchor test

# Unit tests only (no validator needed)
npm run test:unit

# Integration tests (requires validator)
npm run test:integration
```

## Test Results

**Unit Tests: 29/29 PASSING (45ms)**
- Swap calculations (6)
- Price impact (2)
- LP tokens (4)
- Slippage (2)
- Rewards (3)
- Perpetuals (5)
- Edge cases (4)
- Fees (2)
- Constant product (1)

## Helper Functions

### Account Management
- `createAndFundWallet(amount)` - Create funded wallet
- `createTokenMint(decimals)` - Create test token
- `createTokenAccount(mint, owner)` - Create token account
- `airdropTokens(mint, account, amount)` - Fund with tokens

### Calculations
- `calculateSwapOutput()` - AMM swap output
- `calculateInitialLpTokens()` - LP tokens for first deposit
- `calculateLpTokens()` - LP tokens for subsequent deposits
- `calculatePriceImpact()` - Price slippage
- `calculateLiquidationPriceLong/Short()` - Liquidation prices
- `calculateRequiredMargin()` - Margin requirements
- `calculatePendingRewards()` - Reward calculations

### Assertions
- `assertApproxEqual(expected, actual, tolerance)` - Fuzzy comparison
- `verifySlippage(expected, actual, tolerance)` - Slippage check

### PDA Operations
- `findPoolPda(tokenA, tokenB)` - Derive pool PDA
- `findPoolAuthorityPda(tokenA, tokenB)` - Derive authority PDA

### Environment
- `setupTestEnvironment()` - Initialize test env
- `cleanupTestEnvironment()` - Cleanup after tests

## Verified Formulas

### AMM
```
k = x * y
output = (y * input * 9970) / (x * 10000 + input * 9970)
```

### LP Tokens
```
Initial: sqrt(A * B)
Subsequent: min((A * supply) / reserve_A, (B * supply) / reserve_B)
```

### Perpetuals
```
Margin: size / leverage
Liquidation (Long): entry * (1 - 1/leverage)
Liquidation (Short): entry * (1 + 1/leverage)
PnL: ((current - entry) / entry) * size
```

## Test Scaffolds

Ready for implementation:
- swap_operations.test.ts
- reward_system.test.ts
- perpetual_trading.test.ts
- admin_controls.test.ts
- user_journeys.test.ts (e2e)
- attack_vectors.test.ts (security)
- load_tests.test.ts (performance)

## Debugging

```bash
# Verbose output
anchor test -- --verbose

# Skip deploy (reuse programs)
anchor test --skip-deploy

# Specific test file
anchor test --skip-deploy tests/unit/amm_calculations.test.ts

# Watch mode
npm run test:watch
```

## Coverage

Generate coverage reports:
```bash
npm run test:coverage
```

Target: >80% code coverage

## CI/CD

Tests run automatically on:
- Pull requests
- Main branch commits
- Pre-deployment

## Contributing

When adding tests:
1. Use descriptive test names
2. Add comments for complex logic
3. Use helper functions from test-helpers.ts
4. Test both success and failure cases
5. Include edge cases
