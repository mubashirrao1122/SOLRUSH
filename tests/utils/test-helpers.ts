import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

/**
 * Test helper utilities for SolRush DEX testing
 */

export class TestHelpers {
  provider: anchor.AnchorProvider;
  connection: web3.Connection;

  constructor(provider: anchor.AnchorProvider) {
    this.provider = provider;
    this.connection = provider.connection;
  }

  /**
   * Create and fund a new wallet with SOL
   */
  async createAndFundWallet(amountSol: number): Promise<web3.Keypair> {
    const wallet = web3.Keypair.generate();
    
    const signature = await this.connection.requestAirdrop(
      wallet.publicKey,
      amountSol * web3.LAMPORTS_PER_SOL
    );
    
    await this.connection.confirmTransaction(signature, "confirmed");
    
    console.log(`Created and funded wallet: ${wallet.publicKey.toBase58()} with ${amountSol} SOL`);
    return wallet;
  }

  /**
   * Create a token mint
   */
  async createTokenMint(
    decimals: number = 9,
    authority?: web3.Keypair
  ): Promise<web3.PublicKey> {
    const payer = authority || (this.provider.wallet as any).payer;
    
    const mint = await createMint(
      this.connection,
      payer,
      payer.publicKey,
      null,
      decimals
    );
    
    console.log(`Created token mint: ${mint.toBase58()} with ${decimals} decimals`);
    return mint;
  }

  /**
   * Create a token account for a user
   */
  async createTokenAccount(
    mint: web3.PublicKey,
    owner: web3.PublicKey
  ): Promise<web3.PublicKey> {
    const payer = (this.provider.wallet as any).payer;
    
    const account = await createAccount(
      this.connection,
      payer,
      mint,
      owner
    );
    
    console.log(`Created token account: ${account.toBase58()} for owner ${owner.toBase58()}`);
    return account;
  }

  /**
   * Mint tokens to an account
   */
  async airdropTokens(
    mint: web3.PublicKey,
    destination: web3.PublicKey,
    amount: number,
    authority?: web3.Keypair
  ): Promise<void> {
    const payer = authority || (this.provider.wallet as any).payer;
    
    await mintTo(
      this.connection,
      payer,I need comprehensive test suites for my SolRush DEX Solana smart contracts written in Rust using the Anchor framework. Create complete testing code covering all aspects.

## PROJECT CONTEXT
- Platform: Solana blockchain (Devnet)
- Framework: Anchor (Rust)
- Contracts: 6 smart contracts (Liquidity Pool, Swap, Rush Token, Rewards, Perpetual Trading, Admin)
- Pools: SOL/USDC, SOL/wETH, SOL/USDT
- Features: AMM trading, liquidity provision, Rush token rewards (1M cap), leveraged perpetual trading, admin controls

## TESTING REQUIREMENTS

### 1. UNIT TESTS
Create unit tests for mathematical calculations and isolated functions:

**File: tests/unit/amm_calculations.test.ts**
- Test swap output calculation using constant product formula (x * y = k)
- Test with 0.3% fee calculation (997/1000)
- Test price impact calculations for various trade sizes
- Test LP token minting calculations (first deposit and subsequent deposits)
- Test slippage calculations and tolerance checks
- Test edge cases: zero amounts, very small amounts, very large amounts
- Test overflow/underflow protection in all calculations
- Test reward calculation: (userLP / totalLP) * rewardRate * timeElapsed
- Test perpetual margin calculation: positionSize / leverage
- Test liquidation price formulas for long and short positions
- Test that all math operations use checked arithmetic (no panics)

**File: tests/unit/token_operations.test.ts**
- Test Rush token minting respects 1,000,000 cap
- Test token transfer validations
- Test token account creation and closure
- Test SPL token standard compliance
- Test token metadata correctness

### 2. INTEGRATION TESTS
Create integration tests for contract interactions:

**File: tests/integration/pool_operations.test.ts**
Test complete liquidity pool workflows:
- Initialize all three pools (SOL/USDC, SOL/wETH, SOL/USDT)
- Add liquidity to pools and verify LP tokens minted correctly
- Remove liquidity and verify LP tokens burned correctly
- Verify pool reserves update atomically
- Test that fees accumulate in pool correctly
- Test liquidity operations across multiple users
- Test edge cases: removing more than deposited, adding zero liquidity

**File: tests/integration/swap_operations.test.ts**
Test trading functionality:
- Execute swaps in both directions (A→B and B→A) for all three pools
- Verify output amounts match AMM calculations within tolerance
- Test slippage protection (transaction fails if slippage exceeded)
- Verify trading fees (0.3%) are deducted correctly
- Test fee distribution to LP holders
- Verify pool reserves update after each swap
- Test multiple consecutive swaps
- Test swap with insufficient liquidity (should fail gracefully)
- Test swap with incorrect token accounts (should fail)

**File: tests/integration/reward_system.test.ts**
Test Rush token reward distribution:
- Test reward accumulation over time for liquidity providers
- Verify rewards calculated proportionally to LP token holdings
- Test claiming rewards updates user balance correctly
- Test that claimed rewards cannot be claimed again
- Verify total distributed rewards never exceed 1,000,000 cap
- Test reward calculations when user adds/removes liquidity mid-period
- Test rewards for multiple users in same pool
- Test reward rate changes (if applicable)

**File: tests/integration/perpetual_trading.test.ts**
Test leveraged trading:
- Open long positions with various leverage levels (2x-10x)
- Open short positions with various leverage levels
- Verify collateral is locked in escrow correctly
- Test adding margin to existing positions
- Calculate and verify PnL (profit and loss) for positions
- Test position liquidation when price reaches liquidation threshold
- Test that liquidation can be triggered by any user
- Verify collateral returns correctly when closing profitable positions
- Test that positions cannot be opened with insufficient margin
- Test maximum leverage limits are enforced

**File: tests/integration/admin_controls.test.ts**
Test administrative functions:
- Test pause trading functionality (all swaps should fail when paused)
- Test resume trading functionality (swaps work after resume)
- Verify only admin can pause/resume
- Test that existing positions remain safe during pause
- Test fee rate updates by admin (with limits)
- Test admin authority transfer
- Test emergency withdrawal function (only when paused)
- Verify all admin actions emit proper events

### 3. END-TO-END TESTS
Create complete user journey tests:

**File: tests/e2e/user_journeys.test.ts**
Simulate real user workflows:
- **Journey 1: New Trader**
  1. Connect wallet
  2. Execute first swap (SOL → USDC)
  3. Check transaction history
  4. View portfolio dashboard
  5. Verify all balances updated correctly

- **Journey 2: Liquidity Provider**
  1. Add liquidity to SOL/USDC pool
  2. Wait for time to pass (simulate with clock adjustment)
  3. Execute some trades to generate fees
  4. Check accumulated rewards
  5. Claim Rush tokens
  6. Remove partial liquidity
  7. Verify all calculations correct

- **Journey 3: Advanced Trader**
  1. Execute multiple swaps across different pools
  2. Open 5x leveraged long position
  3. Monitor position PnL
  4. Add margin to position
  5. Close position with profit
  6. Verify final balances

- **Journey 4: Emergency Scenario**
  1. Admin pauses trading
  2. User attempts swap (should fail)
  3. Existing positions remain unchanged
  4. Admin resumes trading
  5. User successfully completes swap

### 4. SECURITY TESTS
Create tests for attack vectors and edge cases:

**File: tests/security/attack_vectors.test.ts**
- Test reentrancy protection on all token transfer functions
- Test integer overflow/underflow scenarios (all should fail safely)
- Test unauthorized access attempts (non-admin trying admin functions)
- Test manipulation of pool ratios (should not break invariants)
- Test flash loan attack scenarios
- Test sandwich attack resistance
- Test front-running mitigation (slippage protection)
- Test invalid signer scenarios
- Test incorrect account ownership
- Test PDA derivation with malicious seeds
- Test that contract state remains consistent after failed transactions

**File: tests/security/boundary_tests.test.ts**
- Test with zero amounts (should fail gracefully)
- Test with maximum possible amounts (near u64::MAX)
- Test with minimum possible amounts (1 lamport)
- Test with invalid token mint addresses
- Test with closed accounts
- Test with insufficient balances
- Test with mismatched token accounts
- Test concurrent transactions from same user

### 5. PERFORMANCE TESTS
Create load and stress tests:

**File: tests/performance/load_tests.test.ts**
- Execute 100 concurrent swaps and measure success rate (target: >99%)
- Measure average transaction completion time (target: <2 seconds)
- Test with 1000 sequential operations without degradation
- Measure transaction costs (target: <$0.01 per operation)
- Test pool performance with high TVL (millions in liquidity)
- Test reward calculations with many active users (100+)
- Profile gas/compute unit usage for all operations

### 6. STATE CONSISTENCY TESTS
Verify blockchain state integrity:

**File: tests/state/consistency_tests.test.ts**
- Verify pool K value (x * y) increases only with fees
- Test that total LP supply matches sum of user LP balances
- Verify that pool reserves match token account balances
- Test that total claimed rewards ≤ total minted rewards
- Verify all user balances sum correctly
- Test state consistency after transaction failures
- Test state rollback on errors

## TEST STRUCTURE REQUIREMENTS

Each test file should include:
1. Proper imports and setup
2. Before/after hooks for account setup and cleanup
3. Clear test descriptions using describe() and it() blocks
4. Detailed console.log statements for debugging
5. Comprehensive assertions using expect()
6. Error message validation for failure cases
7. Event emission verification
8. Transaction confirmation checks

## TESTING UTILITIES NEEDED

**File: tests/utils/test-helpers.ts**
Create helper functions:
- createAndFundWallet(amount): Create test wallet with SOL
- createTokenAccount(mint, owner): Create token accounts
- airdropTokens(account, amount): Fund test tokens
- waitForTransaction(signature): Wait for confirmation
- getAccountBalance(account): Check balances
- advanceTime(seconds): Simulate time passage for rewards
- createTestPool(tokenA, tokenB): Initialize test pool
- executeTestSwap(pool, amountIn, minOut): Helper swap function
- calculateExpectedOutput(amountIn, reserveIn, reserveOut): Math helper
- setupTestEnvironment(): Initialize all test accounts
- cleanupTestEnvironment(): Close all test accounts

## EXECUTION COMMANDS NEEDED

Provide scripts for:
1. anchor test - Run all tests on local validator
2. anchor test --skip-local-validator - Run against existing validator
3. npm run test:unit - Run only unit tests
4. npm run test:integration - Run integration tests
5. npm run test:e2e - Run end-to-end tests
6. npm run test:security - Run security tests
7. npm run test:coverage - Generate coverage report
8. anchor test --detach - Run with persistent validator for debugging

## OUTPUT REQUIREMENTS

For each test suite, provide:
1. Complete TypeScript test code ready to run
2. All necessary imports and type definitions
3. Detailed comments explaining each test
4. Console output for debugging
5. Expected success/failure conditions
6. Performance benchmarks where applicable
7. Error handling examples

## VALIDATION CRITERIA

All tests must:
- Use Anchor's testing framework
- Include proper setup and teardown
- Test both success and failure scenarios
- Verify state changes on-chain
- Check event emissions
- Validate error messages
- Be deterministic (no flaky tests)
- Run in under 5 minutes total
- Achieve >80% code coverage
- Pass on both localnet and devnet

## SPECIFIC MATH VALIDATIONS

Include tests that verify:
- Constant product formula: (x + Δx) * (y - Δy) = k (accounting for fees)
- Slippage: |expected - actual| / expected ≤ tolerance
- LP tokens: sqrt(amountA * amountB) for first deposit
- Rewards: (userLP / totalLP) * rate * time
- Liquidation: entryPrice ± (entryPrice / leverage)
- Fees: output = input * 0.997 (0.3% fee)

Generate complete, production-ready test suites that I can run immediately with 'anchor test' command.
      mint,
      destination,
      payer,
      amount
    );
    
    console.log(`Minted ${amount} tokens to ${destination.toBase58()}`);
  }

  /**
   * Wait for transaction confirmation
   */
  async waitForTransaction(signature: string): Promise<void> {
    const latestBlockhash = await this.connection.getLatestBlockhash();
    
    await this.connection.confirmTransaction(
      {
        signature,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      },
      "confirmed"
    );
    
    console.log(`Transaction confirmed: ${signature}`);
  }

  /**
   * Get token account balance
   */
  async getTokenBalance(account: web3.PublicKey): Promise<number> {
    try {
      const accountInfo = await getAccount(this.connection, account);
      return Number(accountInfo.amount);
    } catch (error) {
      console.log(`Error getting balance for ${account.toBase58()}: ${error}`);
      return 0;
    }
  }

  /**
   * Get SOL balance
   */
  async getSolBalance(account: web3.PublicKey): Promise<number> {
    const balance = await this.connection.getBalance(account);
    return balance / web3.LAMPORTS_PER_SOL;
  }

  /**
   * Advance blockchain time (for testing rewards)
   */
  async advanceTime(seconds: number): Promise<void> {
    const currentClock = await this.connection.getBlockTime(
      await this.connection.getSlot()
    );
    
    // On localnet, we can manipulate time using warp
    // This is a placeholder - actual implementation depends on test validator
    console.log(`Advancing time by ${seconds} seconds from ${currentClock}`);
    
    // Sleep for testing purposes
    await new Promise(resolve => setTimeout(resolve, seconds * 1000));
  }

  /**
   * Calculate expected swap output using constant product formula
   */
  calculateSwapOutput(
    amountIn: number,
    reserveIn: number,
    reserveOut: number,
    feeBasisPoints: number = 30
  ): number {
    // Apply fee: amountIn * (10000 - fee) / 10000
    const amountInWithFee = (amountIn * (10000 - feeBasisPoints)) / 10000;
    
    // Constant product: (reserveIn + amountInWithFee) * (reserveOut - amountOut) = k
    // Solve for amountOut: amountOut = (reserveOut * amountInWithFee) / (reserveIn + amountInWithFee)
    const amountOut = (reserveOut * amountInWithFee) / (reserveIn + amountInWithFee);
    
    return Math.floor(amountOut);
  }

  /**
   * Calculate LP tokens for initial deposit
   */
  calculateInitialLpTokens(amountA: number, amountB: number): number {
    return Math.floor(Math.sqrt(amountA * amountB));
  }

  /**
   * Calculate LP tokens for subsequent deposit
   */
  calculateLpTokens(
    amountA: number,
    amountB: number,
    reserveA: number,
    reserveB: number,
    totalLpSupply: number
  ): number {
    const lpFromA = (amountA * totalLpSupply) / reserveA;
    const lpFromB = (amountB * totalLpSupply) / reserveB;
    
    // Use minimum to maintain ratio
    return Math.floor(Math.min(lpFromA, lpFromB));
  }

  /**
   * Calculate price impact percentage
   */
  calculatePriceImpact(
    amountIn: number,
    reserveIn: number,
    reserveOut: number,
    amountOut: number
  ): number {
    const expectedPrice = reserveOut / reserveIn;
    const actualPrice = amountOut / amountIn;
    const impact = ((expectedPrice - actualPrice) / expectedPrice) * 100;
    
    return impact;
  }

  /**
   * Calculate liquidation price for long position
   */
  calculateLiquidationPriceLong(entryPrice: number, leverage: number): number {
    return entryPrice * (1 - 1 / leverage);
  }

  /**
   * Calculate liquidation price for short position
   */
  calculateLiquidationPriceShort(entryPrice: number, leverage: number): number {
    return entryPrice * (1 + 1 / leverage);
  }

  /**
   * Calculate required margin
   */
  calculateRequiredMargin(positionSize: number, leverage: number): number {
    return Math.ceil(positionSize / leverage);
  }

  /**
   * Calculate pending rewards
   */
  calculatePendingRewards(
    userLpBalance: number,
    totalLpSupply: number,
    rewardRate: number,
    timeElapsed: number
  ): number {
    if (totalLpSupply === 0) return 0;
    
    const userShare = userLpBalance / totalLpSupply;
    const rewards = userShare * rewardRate * timeElapsed;
    
    return Math.floor(rewards);
  }

  /**
   * Verify slippage is within tolerance
   */
  verifySlippage(
    expected: number,
    actual: number,
    toleranceBasisPoints: number
  ): boolean {
    const tolerance = (expected * toleranceBasisPoints) / 10000;
    const difference = Math.abs(expected - actual);
    
    console.log(`Expected: ${expected}, Actual: ${actual}, Tolerance: ${tolerance}, Diff: ${difference}`);
    return difference <= tolerance;
  }

  /**
   * Assert values are approximately equal (within tolerance)
   */
  assertApproxEqual(
    actual: number,
    expected: number,
    tolerance: number,
    message?: string
  ): void {
    const diff = Math.abs(actual - expected);
    const isWithinTolerance = diff <= tolerance;
    
    console.log(`Assertion: ${message || 'Values comparison'}`);
    console.log(`  Expected: ${expected}, Actual: ${actual}, Diff: ${diff}, Tolerance: ${tolerance}`);
    
    assert.isTrue(
      isWithinTolerance,
      `${message || 'Values not approximately equal'}: expected ${expected}, got ${actual}, diff ${diff} > tolerance ${tolerance}`
    );
  }

  /**
   * Find PDA for pool
   */
  findPoolPda(
    programId: web3.PublicKey,
    tokenAMint: web3.PublicKey,
    tokenBMint: web3.PublicKey
  ): [web3.PublicKey, number] {
    return web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer(),
      ],
      programId
    );
  }

  /**
   * Find PDA for pool authority
   */
  findPoolAuthorityPda(
    programId: web3.PublicKey,
    tokenAMint: web3.PublicKey,
    tokenBMint: web3.PublicKey
  ): [web3.PublicKey, number] {
    return web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool_authority"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer(),
      ],
      programId
    );
  }

  /**
   * Print test separator for readability
   */
  printTestSeparator(title: string): void {
    console.log("\n" + "=".repeat(80));
    console.log(`  ${title}`);
    console.log("=".repeat(80) + "\n");
  }

  /**
   * Log test step
   */
  logStep(step: number, description: string): void {
    console.log(`\n[Step ${step}] ${description}`);
  }

  /**
   * Log account state for debugging
   */
  async logAccountState(
    label: string,
    account: web3.PublicKey,
    isTokenAccount: boolean = true
  ): Promise<void> {
    if (isTokenAccount) {
      const balance = await this.getTokenBalance(account);
      console.log(`  ${label}: ${account.toBase58()} - Balance: ${balance}`);
    } else {
      const balance = await this.getSolBalance(account);
      console.log(`  ${label}: ${account.toBase58()} - Balance: ${balance} SOL`);
    }
  }

  /**
   * Create test environment with standard setup
   */
  async setupTestEnvironment(): Promise<TestEnvironment> {
    this.printTestSeparator("Setting up test environment");

    // Create test user
    const user = await this.createAndFundWallet(10);

    // Create token mints
    const tokenAMint = await this.createTokenMint(9);
    const tokenBMint = await this.createTokenMint(6); // USDC has 6 decimals
    const tokenCMint = await this.createTokenMint(9);

    // Create user token accounts
    const userTokenA = await this.createTokenAccount(tokenAMint, user.publicKey);
    const userTokenB = await this.createTokenAccount(tokenBMint, user.publicKey);
    const userTokenC = await this.createTokenAccount(tokenCMint, user.publicKey);

    // Fund user with tokens
    await this.airdropTokens(tokenAMint, userTokenA, 1000000 * 10 ** 9); // 1M tokens
    await this.airdropTokens(tokenBMint, userTokenB, 1000000 * 10 ** 6); // 1M USDC
    await this.airdropTokens(tokenCMint, userTokenC, 1000000 * 10 ** 9); // 1M tokens

    console.log("\nTest environment setup complete!");

    return {
      user,
      tokenAMint,
      tokenBMint,
      tokenCMint,
      userTokenA,
      userTokenB,
      userTokenC,
    };
  }

  /**
   * Cleanup test accounts
   */
  async cleanupTestEnvironment(): Promise<void> {
    console.log("\nCleaning up test environment...");
    // Accounts will be cleaned up automatically when validator resets
  }
}

/**
 * Test environment structure
 */
export interface TestEnvironment {
  user: web3.Keypair;
  tokenAMint: web3.PublicKey;
  tokenBMint: web3.PublicKey;
  tokenCMint: web3.PublicKey;
  userTokenA: web3.PublicKey;
  userTokenB: web3.PublicKey;
  userTokenC: web3.PublicKey;
}

/**
 * Pool test data
 */
export interface PoolTestData {
  pool: web3.PublicKey;
  tokenAVault: web3.PublicKey;
  tokenBVault: web3.PublicKey;
  lpMint: web3.PublicKey;
  authority: web3.PublicKey;
}

/**
 * Convert BN to number safely
 */
export function bnToNumber(bn: BN): number {
  return bn.toNumber();
}

/**
 * Convert number to BN
 */
export function numberToBN(num: number): BN {
  return new BN(num);
}
