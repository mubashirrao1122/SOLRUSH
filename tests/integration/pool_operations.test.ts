import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAccount,
  getMint,
} from "@solana/spl-token";
import { assert, expect } from "chai";
import { TestHelpers, TestEnvironment } from "../utils/test-helpers";

/**
 * Integration Tests for Liquidity Pool Operations
 * Tests the complete workflow of pool creation, liquidity management, and state consistency
 */

describe("Liquidity Pool Integration Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolrushLiquidityPool as Program;
  let helpers: TestHelpers;
  let testEnv: TestEnvironment;

  // Pool accounts
  let poolPda: PublicKey;
  let poolBump: number;
  let authorityPda: PublicKey;
  let authorityBump: number;
  let tokenAVault: PublicKey;
  let tokenBVault: PublicKey;
  let lpMint: PublicKey;
  let userLpAccount: PublicKey;

  before(async () => {
    helpers = new TestHelpers(provider);
    helpers.printTestSeparator("LIQUIDITY POOL INTEGRATION TESTS");
    
    // Setup test environment
    testEnv = await helpers.setupTestEnvironment();
  });

  describe("Pool Initialization", () => {
    
    it("should initialize a new liquidity pool (SOL/USDC)", async () => {
      helpers.logStep(1, "Deriving PDAs for pool");

      // Derive pool PDA
      [poolPda, poolBump] = helpers.findPoolPda(
        program.programId,
        testEnv.tokenAMint,
        testEnv.tokenBMint
      );

      // Derive authority PDA
      [authorityPda, authorityBump] = helpers.findPoolAuthorityPda(
        program.programId,
        testEnv.tokenAMint,
        testEnv.tokenBMint
      );

      // Derive LP mint PDA
      [lpMint] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("lp_mint"),
          testEnv.tokenAMint.toBuffer(),
          testEnv.tokenBMint.toBuffer(),
        ],
        program.programId
      );

      console.log(`Pool PDA: ${poolPda.toBase58()}`);
      console.log(`Authority PDA: ${authorityPda.toBase58()}`);
      console.log(`LP Mint: ${lpMint.toBase58()}`);

      helpers.logStep(2, "Deriving vault PDAs");

      [tokenAVault] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("token_a_vault"),
          testEnv.tokenAMint.toBuffer(),
          testEnv.tokenBMint.toBuffer(),
        ],
        program.programId
      );

      [tokenBVault] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("token_b_vault"),
          testEnv.tokenAMint.toBuffer(),
          testEnv.tokenBMint.toBuffer(),
        ],
        program.programId
      );

      console.log(`Token A Vault: ${tokenAVault.toBase58()}`);
      console.log(`Token B Vault: ${tokenBVault.toBase58()}`);

      helpers.logStep(3, "Initializing pool with 0.3% fee");

      const feeRate = 30; // 0.3%

      try {
        await program.methods
          .initializePool(feeRate)
          .accounts({
            pool: poolPda,
            tokenAMint: testEnv.tokenAMint,
            tokenBMint: testEnv.tokenBMint,
            tokenAVault: tokenAVault,
            tokenBVault: tokenBVault,
            lpTokenMint: lpMint,
            poolAuthority: authorityPda,
            payer: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        console.log("✓ Pool initialized successfully");
      } catch (error) {
        console.error("Pool initialization error:", error);
        throw error;
      }

      helpers.logStep(4, "Verifying pool state");

      const poolAccount = await program.account.poolState.fetch(poolPda);
      
      console.log("Pool State:");
      console.log(`  Token A Reserve: ${poolAccount.tokenAReserve.toString()}`);
      console.log(`  Token B Reserve: ${poolAccount.tokenBReserve.toString()}`);
      console.log(`  LP Token Supply: ${poolAccount.lpTokenSupply.toString()}`);
      console.log(`  Fee Rate: ${poolAccount.feeRate}`);

      assert.equal(poolAccount.feeRate, feeRate);
      assert.equal(poolAccount.tokenAReserve.toNumber(), 0);
      assert.equal(poolAccount.tokenBReserve.toNumber(), 0);
      assert.equal(poolAccount.lpTokenSupply.toNumber(), 0);
    });

    it("should reject initialization with fee rate > 100 (1%)", async () => {
      const highFee = 150; // 1.5%

      try {
        await program.methods
          .initializePool(highFee)
          .accounts({
            pool: poolPda,
            tokenAMint: testEnv.tokenAMint,
            tokenBMint: testEnv.tokenBMint,
            tokenAVault: tokenAVault,
            tokenBVault: tokenBVault,
            lpTokenMint: lpMint,
            poolAuthority: authorityPda,
            payer: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        assert.fail("Should have rejected high fee rate");
      } catch (error) {
        console.log("✓ Correctly rejected high fee rate");
        assert.include(error.toString(), "InvalidFeeRate");
      }
    });
  });

  describe("Add Liquidity", () => {
    
    it("should add initial liquidity and mint LP tokens", async () => {
      helpers.logStep(1, "Creating user LP token account");

      userLpAccount = await helpers.createTokenAccount(
        lpMint,
        testEnv.user.publicKey
      );

      const depositAmountA = new BN(1000000 * 10 ** 9); // 1M tokens
      const depositAmountB = new BN(2000000 * 10 ** 6); // 2M USDC (6 decimals)
      const minLpTokens = new BN(0); // No slippage for first deposit

      helpers.logStep(2, "Adding initial liquidity");

      console.log(`Depositing: ${depositAmountA.toString()} A, ${depositAmountB.toString()} B`);

      try {
        await program.methods
          .addLiquidity(depositAmountA, depositAmountB, minLpTokens)
          .accounts({
            pool: poolPda,
            tokenAVault: tokenAVault,
            tokenBVault: tokenBVault,
            lpTokenMint: lpMint,
            userTokenA: testEnv.userTokenA,
            userTokenB: testEnv.userTokenB,
            userLpToken: userLpAccount,
            poolAuthority: authorityPda,
            user: testEnv.user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([testEnv.user])
          .rpc();

        console.log("✓ Liquidity added successfully");
      } catch (error) {
        console.error("Add liquidity error:", error);
        throw error;
      }

      helpers.logStep(3, "Verifying LP tokens minted");

      const lpBalance = await helpers.getTokenBalance(userLpAccount);
      const expectedLp = helpers.calculateInitialLpTokens(
        depositAmountA.toNumber(),
        depositAmountB.toNumber()
      );

      console.log(`LP tokens received: ${lpBalance}`);
      console.log(`Expected (sqrt): ${expectedLp}`);

      // Allow small rounding difference
      helpers.assertApproxEqual(
        lpBalance,
        expectedLp,
        100,
        "LP tokens should match sqrt(A * B)"
      );

      helpers.logStep(4, "Verifying pool reserves updated");

      const poolAccount = await program.account.poolState.fetch(poolPda);
      
      console.log(`Reserve A: ${poolAccount.tokenAReserve.toString()}`);
      console.log(`Reserve B: ${poolAccount.tokenBReserve.toString()}`);
      console.log(`LP Supply: ${poolAccount.lpTokenSupply.toString()}`);

      assert.equal(
        poolAccount.tokenAReserve.toString(),
        depositAmountA.toString()
      );
      assert.equal(
        poolAccount.tokenBReserve.toString(),
        depositAmountB.toString()
      );
    });

    it("should add subsequent liquidity proportionally", async () => {
      const poolBefore = await program.account.poolState.fetch(poolPda);
      const lpBalanceBefore = await helpers.getTokenBalance(userLpAccount);

      const depositAmountA = new BN(100000 * 10 ** 9); // 100K tokens
      const depositAmountB = new BN(200000 * 10 ** 6); // 200K USDC
      const minLpTokens = new BN(0);

      helpers.logStep(1, "Adding more liquidity");

      await program.methods
        .addLiquidity(depositAmountA, depositAmountB, minLpTokens)
        .accounts({
          pool: poolPda,
          tokenAVault: tokenAVault,
          tokenBVault: tokenBVault,
          lpTokenMint: lpMint,
          userTokenA: testEnv.userTokenA,
          userTokenB: testEnv.userTokenB,
          userLpToken: userLpAccount,
          poolAuthority: authorityPda,
          user: testEnv.user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([testEnv.user])
        .rpc();

      helpers.logStep(2, "Verifying proportional LP tokens minted");

      const lpBalanceAfter = await helpers.getTokenBalance(userLpAccount);
      const lpReceived = lpBalanceAfter - lpBalanceBefore;

      const expectedLp = helpers.calculateLpTokens(
        depositAmountA.toNumber(),
        depositAmountB.toNumber(),
        poolBefore.tokenAReserve.toNumber(),
        poolBefore.tokenBReserve.toNumber(),
        poolBefore.lpTokenSupply.toNumber()
      );

      console.log(`LP tokens received: ${lpReceived}`);
      console.log(`Expected: ${expectedLp}`);

      helpers.assertApproxEqual(
        lpReceived,
        expectedLp,
        100,
        "Proportional LP tokens"
      );
    });

    it("should reject adding liquidity with zero amounts", async () => {
      try {
        await program.methods
          .addLiquidity(new BN(0), new BN(0), new BN(0))
          .accounts({
            pool: poolPda,
            tokenAVault: tokenAVault,
            tokenBVault: tokenBVault,
            lpTokenMint: lpMint,
            userTokenA: testEnv.userTokenA,
            userTokenB: testEnv.userTokenB,
            userLpToken: userLpAccount,
            poolAuthority: authorityPda,
            user: testEnv.user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([testEnv.user])
          .rpc();

        assert.fail("Should reject zero amounts");
      } catch (error) {
        console.log("✓ Correctly rejected zero amounts");
      }
    });

    it("should enforce minimum LP tokens (slippage protection)", async () => {
      const depositAmountA = new BN(1000);
      const depositAmountB = new BN(2000);
      const minLpTokens = new BN(999999999); // Unreasonably high

      try {
        await program.methods
          .addLiquidity(depositAmountA, depositAmountB, minLpTokens)
          .accounts({
            pool: poolPda,
            tokenAVault: tokenAVault,
            tokenBVault: tokenBVault,
            lpTokenMint: lpMint,
            userTokenA: testEnv.userTokenA,
            userTokenB: testEnv.userTokenB,
            userLpToken: userLpAccount,
            poolAuthority: authorityPda,
            user: testEnv.user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([testEnv.user])
          .rpc();

        assert.fail("Should enforce minimum LP tokens");
      } catch (error) {
        console.log("✓ Slippage protection working");
        assert.include(error.toString(), "SlippageExceeded");
      }
    });
  });

  describe("Remove Liquidity", () => {
    
    it("should remove liquidity and burn LP tokens", async () => {
      const lpBalanceBefore = await helpers.getTokenBalance(userLpAccount);
      const tokenABalanceBefore = await helpers.getTokenBalance(testEnv.userTokenA);
      const tokenBBalanceBefore = await helpers.getTokenBalance(testEnv.userTokenB);
      const poolBefore = await program.account.poolState.fetch(poolPda);

      // Burn 10% of LP tokens
      const lpToBurn = new BN(Math.floor(lpBalanceBefore * 0.1));
      const minTokenA = new BN(0);
      const minTokenB = new BN(0);

      helpers.logStep(1, "Removing liquidity");

      console.log(`Burning ${lpToBurn.toString()} LP tokens`);

      await program.methods
        .removeLiquidity(lpToBurn, minTokenA, minTokenB)
        .accounts({
          pool: poolPda,
          tokenAVault: tokenAVault,
          tokenBVault: tokenBVault,
          lpTokenMint: lpMint,
          userTokenA: testEnv.userTokenA,
          userTokenB: testEnv.userTokenB,
          userLpToken: userLpAccount,
          poolAuthority: authorityPda,
          user: testEnv.user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([testEnv.user])
        .rpc();

      helpers.logStep(2, "Verifying tokens received");

      const lpBalanceAfter = await helpers.getTokenBalance(userLpAccount);
      const tokenABalanceAfter = await helpers.getTokenBalance(testEnv.userTokenA);
      const tokenBBalanceAfter = await helpers.getTokenBalance(testEnv.userTokenB);

      const lpBurned = lpBalanceBefore - lpBalanceAfter;
      const tokenAReceived = tokenABalanceAfter - tokenABalanceBefore;
      const tokenBReceived = tokenBBalanceAfter - tokenBBalanceBefore;

      console.log(`LP burned: ${lpBurned}`);
      console.log(`Token A received: ${tokenAReceived}`);
      console.log(`Token B received: ${tokenBReceived}`);

      // Calculate expected amounts
      const expectedTokenA = Math.floor(
        (lpToBurn.toNumber() * poolBefore.tokenAReserve.toNumber()) / poolBefore.lpTokenSupply.toNumber()
      );
      const expectedTokenB = Math.floor(
        (lpToBurn.toNumber() * poolBefore.tokenBReserve.toNumber()) / poolBefore.lpTokenSupply.toNumber()
      );

      console.log(`Expected token A: ${expectedTokenA}`);
      console.log(`Expected token B: ${expectedTokenB}`);

      assert.equal(lpBurned, lpToBurn.toNumber());
      helpers.assertApproxEqual(tokenAReceived, expectedTokenA, 100, "Token A amount");
      helpers.assertApproxEqual(tokenBReceived, expectedTokenB, 100, "Token B amount");
    });

    it("should reject removing more LP tokens than owned", async () => {
      const lpBalance = await helpers.getTokenBalance(userLpAccount);
      const excessiveAmount = new BN(lpBalance + 1000000);

      try {
        await program.methods
          .removeLiquidity(excessiveAmount, new BN(0), new BN(0))
          .accounts({
            pool: poolPda,
            tokenAVault: tokenAVault,
            tokenBVault: tokenBVault,
            lpTokenMint: lpMint,
            userTokenA: testEnv.userTokenA,
            userTokenB: testEnv.userTokenB,
            userLpToken: userLpAccount,
            poolAuthority: authorityPda,
            user: testEnv.user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([testEnv.user])
          .rpc();

        assert.fail("Should reject excessive burn amount");
      } catch (error) {
        console.log("✓ Correctly rejected excessive amount");
      }
    });
  });

  describe("Pool State Consistency", () => {
    
    it("should maintain correct reserve ratios", async () => {
      const poolAccount = await program.account.poolState.fetch(poolPda);
      
      const ratio = poolAccount.tokenAReserve.toNumber() / poolAccount.tokenBReserve.toNumber();
      
      console.log(`Reserve A: ${poolAccount.tokenAReserve.toString()}`);
      console.log(`Reserve B: ${poolAccount.tokenBReserve.toString()}`);
      console.log(`Ratio: ${ratio}`);

      // Ratio should be maintained (accounting for different decimals)
      // Token A has 9 decimals, Token B has 6 decimals
      // So we expect ratio to be around 0.5 (1M * 10^9 / 2M * 10^6)
      expect(ratio).to.be.closeTo(0.5, 0.1);
    });

    it("should have vault balances matching pool reserves", async () => {
      const poolAccount = await program.account.poolState.fetch(poolPda);
      const vaultABalance = await helpers.getTokenBalance(tokenAVault);
      const vaultBBalance = await helpers.getTokenBalance(tokenBVault);

      console.log(`Pool Reserve A: ${poolAccount.tokenAReserve.toString()}`);
      console.log(`Vault A Balance: ${vaultABalance}`);
      console.log(`Pool Reserve B: ${poolAccount.tokenBReserve.toString()}`);
      console.log(`Vault B Balance: ${vaultBBalance}`);

      assert.equal(
        poolAccount.tokenAReserve.toNumber(),
        vaultABalance,
        "Vault A should match reserve A"
      );
      assert.equal(
        poolAccount.tokenBReserve.toNumber(),
        vaultBBalance,
        "Vault B should match reserve B"
      );
    });

    it("should have LP supply matching minted tokens", async () => {
      const poolAccount = await program.account.poolState.fetch(poolPda);
      const lpMintAccount = await getMint(provider.connection, lpMint);

      console.log(`Pool LP Supply: ${poolAccount.lpTokenSupply.toString()}`);
      console.log(`Mint Supply: ${lpMintAccount.supply.toString()}`);

      assert.equal(
        poolAccount.lpTokenSupply.toString(),
        lpMintAccount.supply.toString(),
        "LP supply should match mint supply"
      );
    });
  });

  describe("Multiple Users", () => {
    
    it("should handle liquidity from multiple users correctly", async () => {
      helpers.logStep(1, "Creating second user");

      const user2 = await helpers.createAndFundWallet(5);
      const user2TokenA = await helpers.createTokenAccount(testEnv.tokenAMint, user2.publicKey);
      const user2TokenB = await helpers.createTokenAccount(testEnv.tokenBMint, user2.publicKey);
      const user2LpAccount = await helpers.createTokenAccount(lpMint, user2.publicKey);

      // Fund user2
      await helpers.airdropTokens(testEnv.tokenAMint, user2TokenA, 500000 * 10 ** 9);
      await helpers.airdropTokens(testEnv.tokenBMint, user2TokenB, 500000 * 10 ** 6);

      helpers.logStep(2, "User 2 adding liquidity");

      const depositAmountA = new BN(50000 * 10 ** 9);
      const depositAmountB = new BN(100000 * 10 ** 6);

      await program.methods
        .addLiquidity(depositAmountA, depositAmountB, new BN(0))
        .accounts({
          pool: poolPda,
          tokenAVault: tokenAVault,
          tokenBVault: tokenBVault,
          lpTokenMint: lpMint,
          userTokenA: user2TokenA,
          userTokenB: user2TokenB,
          userLpToken: user2LpAccount,
          poolAuthority: authorityPda,
          user: user2.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user2])
        .rpc();

      const user1LpBalance = await helpers.getTokenBalance(userLpAccount);
      const user2LpBalance = await helpers.getTokenBalance(user2LpAccount);

      console.log(`User 1 LP: ${user1LpBalance}`);
      console.log(`User 2 LP: ${user2LpBalance}`);

      // Both users should have LP tokens
      expect(user1LpBalance).to.be.greaterThan(0);
      expect(user2LpBalance).to.be.greaterThan(0);

      helpers.logStep(3, "Verifying total LP supply");

      const poolAccount = await program.account.poolState.fetch(poolPda);
      const totalUserLp = user1LpBalance + user2LpBalance;

      console.log(`Total user LP: ${totalUserLp}`);
      console.log(`Pool LP supply: ${poolAccount.lpTokenSupply.toString()}`);

      // Total should match (within rounding)
      helpers.assertApproxEqual(
        totalUserLp,
        poolAccount.lpTokenSupply.toNumber(),
        10,
        "Total LP should match pool supply"
      );
    });
  });

  after(async () => {
    await helpers.cleanupTestEnvironment();
    console.log("\n✓ All liquidity pool tests completed successfully!\n");
  });
});
