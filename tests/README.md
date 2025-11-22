# SolRush DEX Test Suite

Comprehensive testing suite for the SolRush decentralized exchange on Solana.

## Test Structure

```
tests/
├── utils/
│   └── test-helpers.ts          # Shared utilities and helper functions
├── unit/
│   ├── amm_calculations.test.ts # Pure mathematical calculations
│   └── token_operations.test.ts # Token logic tests
├── integration/
│   ├── pool_operations.test.ts  # Liquidity pool workflows
│   ├── swap_operations.test.ts  # Trading functionality
│   ├── reward_system.test.ts    # Reward distribution
│   ├── perpetual_trading.test.ts# Leveraged positions
│   └── admin_controls.test.ts   # Administrative functions
├── e2e/
│   └── user_journeys.test.ts    # Complete user workflows
├── security/
│   ├── attack_vectors.test.ts   # Security tests
│   └── boundary_tests.test.ts   # Edge cases
└── performance/
    └── load_tests.test.ts       # Performance benchmarks
```

## Prerequisites

Install dependencies:
```bash
yarn install
```

Build programs:
```bash
anchor build
```

## Running Tests

### Run All Tests
```bash
anchor test
```

### Run Specific Test Suites

**Unit Tests Only** (No blockchain needed):
```bash
npm run test:unit
```

**Integration Tests** (Requires local validator):
```bash
npm run test:integration
```

**End-to-End Tests**:
```bash
npm run test:e2e
```

**Security Tests**:
```bash
npm run test:security
```

**All Tests with Coverage**:
```bash
npm run test:coverage
```

### Run Individual Test Files
```bash
# Run specific test file
anchor test --skip-deploy tests/integration/pool_operations.test.ts

# Run with verbose output
anchor test -- --verbose
```

## Test Categories

### 1. Unit Tests (`tests/unit/`)

Pure function tests without blockchain interaction.

**AMM Calculations** (`amm_calculations.test.ts`):
- Swap output calculation (constant product formula)
- Price impact calculations
- LP token minting/burning math
- Slippage validation
- Reward calculations
- Perpetual trading math (margin, liquidation prices, PnL)
- Fee calculations
- Overflow protection

**Token Operations** (`token_operations.test.ts`):
- Rush token supply cap enforcement
- Token transfer validations
- Account creation/closure
- SPL token compliance

### 2. Integration Tests (`tests/integration/`)

Tests program instructions and state management.

**Pool Operations** (`pool_operations.test.ts`):
- Pool initialization for all pairs (SOL/USDC, SOL/wETH, SOL/USDT)
- Adding liquidity (initial and subsequent)
- Removing liquidity
- LP token minting/burning
- Multiple user scenarios
- State consistency checks

**Swap Operations** (`swap_operations.test.ts`):
- Bidirectional swaps (A→B, B→A)
- Fee deduction and distribution
- Slippage protection
- Price impact validation
- Reserve updates
- Multiple consecutive swaps

**Reward System** (`reward_system.test.ts`):
- Reward accumulation over time
- Proportional distribution
- Claiming rewards
- Supply cap respect
- Multiple pool rewards
- User reward tracking

**Perpetual Trading** (`perpetual_trading.test.ts`):
- Opening long/short positions
- Leverage validation (2x-10x)
- Collateral management
- Position liquidation
- PnL calculations
- Margin additions

**Admin Controls** (`admin_controls.test.ts`):
- Pause/resume trading
- Fee rate updates
- Admin authority transfer
- Emergency withdrawals
- Event emissions

### 3. End-to-End Tests (`tests/e2e/`)

Complete user workflows from start to finish.

**User Journeys** (`user_journeys.test.ts`):
- New trader workflow
- Liquidity provider journey
- Advanced trader with leverage
- Emergency pause scenario
- Multi-pool operations

### 4. Security Tests (`tests/security/`)

Attack vectors and vulnerability testing.

**Attack Vectors** (`attack_vectors.test.ts`):
- Reentrancy protection
- Integer overflow/underflow
- Unauthorized access attempts
- Pool manipulation attempts
- Flash loan simulations
- Front-running mitigation
- Invalid signer scenarios
- PDA derivation attacks

**Boundary Tests** (`boundary_tests.test.ts`):
- Zero amount handling
- Maximum value testing (u64::MAX)
- Minimum value testing
- Invalid addresses
- Closed accounts
- Insufficient balances
- Concurrent transactions

### 5. Performance Tests (`tests/performance/`)

Load testing and benchmarking.

**Load Tests** (`load_tests.test.ts`):
- 100 concurrent swaps
- Transaction completion time
- 1000 sequential operations
- Gas cost measurements
- High TVL performance
- Many active users

## Test Helpers

The `test-helpers.ts` file provides utilities:

### Account Management
```typescript
helpers.createAndFundWallet(amountSol)
helpers.createTokenMint(decimals)
helpers.createTokenAccount(mint, owner)
helpers.airdropTokens(mint, destination, amount)
```

### Balance Checks
```typescript
helpers.getTokenBalance(account)
helpers.getSolBalance(account)
```

### Math Helpers
```typescript
helpers.calculateSwapOutput(amountIn, reserveIn, reserveOut, fee)
helpers.calculateInitialLpTokens(amountA, amountB)
helpers.calculateLpTokens(amountA, amountB, reserveA, reserveB, totalLp)
helpers.calculatePriceImpact(amountIn, reserveIn, reserveOut, amountOut)
helpers.calculateLiquidationPriceLong(entryPrice, leverage)
helpers.calculateLiquidationPriceShort(entryPrice, leverage)
helpers.calculateRequiredMargin(positionSize, leverage)
helpers.calculatePendingRewards(userLp, totalLp, rate, time)
```

### Assertions
```typescript
helpers.assertApproxEqual(actual, expected, tolerance, message)
helpers.verifySlippage(expected, actual, toleranceBps)
```

### PDA Derivation
```typescript
helpers.findPoolPda(programId, tokenAMint, tokenBMint)
helpers.findPoolAuthorityPda(programId, tokenAMint, tokenBMint)
```

### Environment
```typescript
const testEnv = await helpers.setupTestEnvironment()
await helpers.cleanupTestEnvironment()
```

## Writing New Tests

### Unit Test Template
```typescript
import { describe, it } from "mocha";
import { expect } from "chai";

describe("Feature Name", () => {
  it("should behave correctly", () => {
    // Arrange
    const input = 100;
    
    // Act
    const result = calculateSomething(input);
    
    // Assert
    expect(result).to.equal(expected);
  });
});
```

### Integration Test Template
```typescript
import * as anchor from "@coral-xyz/anchor";
import { TestHelpers } from "../utils/test-helpers";

describe("Program Feature", () => {
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.ProgramName;
  let helpers: TestHelpers;

  before(async () => {
    helpers = new TestHelpers(provider);
    // Setup
  });

  it("should execute instruction correctly", async () => {
    // Test implementation
  });

  after(async () => {
    // Cleanup
  });
});
```

## Test Coverage Goals

- **Unit Tests**: >95% coverage of pure functions
- **Integration Tests**: All instructions tested
- **E2E Tests**: All user workflows covered
- **Security Tests**: All known attack vectors tested
- **Performance**: Benchmarks for critical paths

## Expected Test Results

All tests should pass with:
- No compilation errors
- No runtime panics
- Consistent state after each test
- Total execution time < 5 minutes
- >80% overall code coverage

## Debugging Tests

### Verbose Output
```bash
anchor test -- --verbose
```

### Run Specific Test
```bash
anchor test --skip-deploy tests/integration/pool_operations.test.ts -- --grep "should initialize"
```

### Keep Validator Running
```bash
anchor test --detach
```

Then in another terminal:
```bash
anchor test --skip-local-validator
```

### View Logs
```bash
solana logs
```

## Common Issues

### Issue: Tests Timeout
**Solution**: Increase Mocha timeout in test file:
```typescript
this.timeout(60000); // 60 seconds
```

### Issue: Insufficient SOL
**Solution**: Fund wallets with more SOL:
```typescript
await helpers.createAndFundWallet(10); // 10 SOL
```

### Issue: Account Already Exists
**Solution**: Use `anchor test` without `--skip-deploy` to reset validator

### Issue: Transaction Simulation Failed
**Solution**: Check account ownership and PDA derivations

## Continuous Integration

Tests are designed to run in CI/CD pipelines:

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Solana
        run: sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
      - name: Install Anchor
        run: cargo install --git https://github.com/coral-xyz/anchor avm
      - name: Install Dependencies
        run: yarn install
      - name: Build
        run: anchor build
      - name: Test
        run: anchor test
```

## Test Data

All tests use deterministic test data for reproducibility:
- Token A: 9 decimals (SOL-like)
- Token B: 6 decimals (USDC-like)
- Token C: 9 decimals (wETH-like)
- Initial liquidity: 1M / 2M ratio
- Fee rate: 30 basis points (0.3%)
- Max slippage: 1000 basis points (10%)

## Contributing

When adding new features:
1. Write unit tests first
2. Add integration tests for new instructions
3. Update E2E tests if user workflow changes
4. Add security tests for new attack surfaces
5. Ensure all tests pass before submitting PR

## Support

For test-related issues:
1. Check test output for error messages
2. Review Solana program logs
3. Verify account states with `solana account`
4. Consult Anchor documentation
5. Open an issue on GitHub
