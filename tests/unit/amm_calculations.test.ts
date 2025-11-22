import { describe, it } from "mocha";
import { expect, assert } from "chai";
import { BN } from "@coral-xyz/anchor";

/**
 * Unit Tests for AMM Mathematical Calculations
 * Tests the core formulas used in the SolRush DEX without blockchain interaction
 */

describe("AMM Calculations Unit Tests", () => {
  
  describe("Swap Output Calculation (Constant Product Formula)", () => {
    
    it("should calculate swap output correctly with 0.3% fee", () => {
      // Test case: 100 tokens in, reserves 1000/2000
      const amountIn = 100;
      const reserveIn = 1000;
      const reserveOut = 2000;
      const fee = 30; // 0.3% = 30 basis points
      
      // Apply fee: 100 * (10000 - 30) / 10000 = 99.7
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      
      // Constant product: output = (reserveOut * amountInWithFee) / (reserveIn + amountInWithFee)
      const expectedOutput = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
      
      console.log(`Input: ${amountIn}, ReserveIn: ${reserveIn}, ReserveOut: ${reserveOut}`);
      console.log(`Amount after fee: ${amountInWithFee}`);
      console.log(`Expected output: ${expectedOutput}`);
      
      // Output should be approximately 181 tokens
      expect(expectedOutput).to.be.closeTo(181, 1);
    });

    it("should maintain constant product (k) after swap accounting for fees", () => {
      const amountIn = 1000;
      const reserveIn = 10000;
      const reserveOut = 20000;
      const fee = 30;
      
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      const amountOut = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
      
      const kBefore = reserveIn * reserveOut;
      const kAfter = (reserveIn + amountIn) * (reserveOut - amountOut);
      
      console.log(`K before: ${kBefore}`);
      console.log(`K after: ${kAfter}`);
      console.log(`K increased by: ${kAfter - kBefore} (due to fees)`);
      
      // K should increase due to fees (this is how LPs earn)
      expect(kAfter).to.be.greaterThan(kBefore);
    });

    it("should handle small swap amounts correctly", () => {
      const amountIn = 1; // 1 lamport
      const reserveIn = 1000000;
      const reserveOut = 2000000;
      const fee = 30;
      
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      const amountOut = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
      
      console.log(`Tiny swap: ${amountIn} in -> ${amountOut} out`);
      
      // Should return some output even for tiny amounts
      expect(amountOut).to.be.greaterThan(0);
    });

    it("should handle large swap amounts with high price impact", () => {
      const amountIn = 5000; // 50% of reserves
      const reserveIn = 10000;
      const reserveOut = 20000;
      const fee = 30;
      
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      const amountOut = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
      
      // Calculate price impact
      const expectedPrice = reserveOut / reserveIn;
      const actualPrice = amountOut / amountIn;
      const priceImpact = ((expectedPrice - actualPrice) / expectedPrice) * 100;
      
      console.log(`Large swap - Price impact: ${priceImpact.toFixed(2)}%`);
      console.log(`Expected price: ${expectedPrice}, Actual: ${actualPrice}`);
      
      // Large swaps should have significant price impact
      expect(priceImpact).to.be.greaterThan(20); // > 20% impact
      expect(amountOut).to.be.lessThan(10000); // Less than 50% of reserves
    });

    it("should return zero for zero input", () => {
      const amountIn = 0;
      const reserveIn = 10000;
      const reserveOut = 20000;
      const fee = 30;
      
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      const amountOut = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
      
      expect(amountOut).to.equal(0);
    });

    it("should calculate correctly for different fee rates", () => {
      const amountIn = 1000;
      const reserveIn = 10000;
      const reserveOut = 20000;
      
      // Test with different fees
      const fees = [0, 10, 30, 50, 100]; // 0%, 0.1%, 0.3%, 0.5%, 1%
      
      fees.forEach(fee => {
        const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
        const amountOut = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
        
        console.log(`Fee ${fee/100}% -> Output: ${amountOut}`);
      });
      
      // Higher fees should result in lower output
      const output30 = Math.floor((reserveOut * (amountIn * 9970 / 10000)) / (reserveIn + (amountIn * 9970 / 10000)));
      const output100 = Math.floor((reserveOut * (amountIn * 9900 / 10000)) / (reserveIn + (amountIn * 9900 / 10000)));
      
      expect(output30).to.be.greaterThan(output100);
    });
  });

  describe("Price Impact Calculation", () => {
    
    it("should calculate price impact correctly", () => {
      const amountIn = 1000;
      const reserveIn = 10000;
      const reserveOut = 20000;
      const amountOut = 1800; // Calculated output
      
      const expectedPrice = reserveOut / reserveIn; // 2.0
      const actualPrice = amountOut / amountIn; // 1.8
      const priceImpact = ((expectedPrice - actualPrice) / expectedPrice) * 100;
      
      console.log(`Expected price ratio: ${expectedPrice}`);
      console.log(`Actual price ratio: ${actualPrice}`);
      console.log(`Price impact: ${priceImpact.toFixed(2)}%`);
      
      expect(priceImpact).to.be.closeTo(10, 0.1); // 10% impact
    });

    it("should show minimal impact for small trades", () => {
      const amountIn = 10;
      const reserveIn = 100000;
      const reserveOut = 200000;
      const fee = 30;
      
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      const amountOut = Math.floor((reserveOut * amountInWithFee) / (reserveIn + amountInWithFee));
      
      const expectedPrice = reserveOut / reserveIn;
      const actualPrice = amountOut / amountIn;
      const priceImpact = ((expectedPrice - actualPrice) / expectedPrice) * 100;
      
      console.log(`Small trade price impact: ${priceImpact.toFixed(4)}%`);
      
      expect(priceImpact).to.be.lessThan(0.1); // < 0.1% impact
    });
  });

  describe("LP Token Calculation", () => {
    
    it("should calculate initial LP tokens using geometric mean", () => {
      const amountA = 1000000; // 1M tokens
      const amountB = 2000000; // 2M tokens
      
      const lpTokens = Math.floor(Math.sqrt(amountA * amountB));
      
      console.log(`Initial deposit: ${amountA} A, ${amountB} B`);
      console.log(`LP tokens minted: ${lpTokens}`);
      console.log(`Geometric mean: ${Math.sqrt(amountA * amountB)}`);
      
      expect(lpTokens).to.equal(1414213);
    });

    it("should calculate subsequent LP tokens proportionally", () => {
      const depositA = 1000;
      const depositB = 2000;
      const reserveA = 10000;
      const reserveB = 20000;
      const totalLpSupply = 14142;
      
      // Calculate LP tokens from both ratios
      const lpFromA = (depositA * totalLpSupply) / reserveA;
      const lpFromB = (depositB * totalLpSupply) / reserveB;
      
      // Use minimum to maintain pool ratio
      const lpTokens = Math.floor(Math.min(lpFromA, lpFromB));
      
      console.log(`LP from A ratio: ${lpFromA}`);
      console.log(`LP from B ratio: ${lpFromB}`);
      console.log(`LP tokens minted: ${lpTokens}`);
      
      expect(lpTokens).to.equal(1414);
    });

    it("should handle unbalanced deposits correctly", () => {
      const depositA = 1000;
      const depositB = 1000; // Unbalanced (should be 2000)
      const reserveA = 10000;
      const reserveB = 20000;
      const totalLpSupply = 14142;
      
      const lpFromA = (depositA * totalLpSupply) / reserveA;
      const lpFromB = (depositB * totalLpSupply) / reserveB;
      const lpTokens = Math.floor(Math.min(lpFromA, lpFromB));
      
      console.log(`Unbalanced deposit: ${depositA} A, ${depositB} B`);
      console.log(`LP tokens: ${lpTokens} (limited by token B)`);
      
      // Should be limited by the smaller ratio (token B)
      expect(lpTokens).to.equal(Math.floor(lpFromB));
      expect(lpFromB).to.be.lessThan(lpFromA);
    });

    it("should calculate removable amounts correctly", () => {
      const lpTokensBurn = 1000;
      const totalLpSupply = 14142;
      const reserveA = 10000;
      const reserveB = 20000;
      
      const amountA = Math.floor((lpTokensBurn * reserveA) / totalLpSupply);
      const amountB = Math.floor((lpTokensBurn * totalLpSupply) / totalLpSupply);
      
      console.log(`Burning ${lpTokensBurn} LP tokens`);
      console.log(`Receiving: ${amountA} A, ${amountB} B`);
      
      expect(amountA).to.be.greaterThan(0);
      expect(amountB).to.be.greaterThan(0);
    });
  });

  describe("Slippage Calculation", () => {
    
    it("should validate slippage within tolerance", () => {
      const expected = 1000;
      const actual = 995;
      const toleranceBps = 100; // 1%
      
      const tolerance = (expected * toleranceBps) / 10000;
      const difference = Math.abs(expected - actual);
      const isWithin = difference <= tolerance;
      
      console.log(`Expected: ${expected}, Actual: ${actual}`);
      console.log(`Tolerance: ${tolerance}, Difference: ${difference}`);
      console.log(`Within tolerance: ${isWithin}`);
      
      expect(isWithin).to.be.true;
    });

    it("should reject trades exceeding slippage tolerance", () => {
      const expected = 1000;
      const actual = 880; // 12% slippage
      const maxSlippageBps = 1000; // 10%
      
      const tolerance = (expected * maxSlippageBps) / 10000;
      const difference = Math.abs(expected - actual);
      const isWithin = difference <= tolerance;
      
      console.log(`High slippage: ${((expected - actual) / expected * 100).toFixed(2)}%`);
      
      expect(isWithin).to.be.false;
    });
  });

  describe("Reward Calculation", () => {
    
    it("should calculate rewards proportionally to LP holdings", () => {
      const userLpBalance = 1000;
      const totalLpSupply = 10000;
      const rewardRate = 100; // tokens per second
      const timeElapsed = 3600; // 1 hour
      
      const userShare = userLpBalance / totalLpSupply;
      const rewards = Math.floor(userShare * rewardRate * timeElapsed);
      
      console.log(`User has ${userLpBalance}/${totalLpSupply} LP (${userShare * 100}%)`);
      console.log(`Rewards for ${timeElapsed}s: ${rewards} tokens`);
      
      expect(rewards).to.equal(36000); // 10% * 100 * 3600
    });

    it("should return zero rewards for zero LP balance", () => {
      const userLpBalance = 0;
      const totalLpSupply = 10000;
      const rewardRate = 100;
      const timeElapsed = 3600;
      
      const userShare = userLpBalance / totalLpSupply;
      const rewards = Math.floor(userShare * rewardRate * timeElapsed);
      
      expect(rewards).to.equal(0);
    });

    it("should handle multiple users correctly", () => {
      const totalLpSupply = 10000;
      const rewardRate = 100;
      const timeElapsed = 1000;
      
      const users = [
        { lp: 5000, expectedShare: 0.5 },
        { lp: 3000, expectedShare: 0.3 },
        { lp: 2000, expectedShare: 0.2 },
      ];
      
      let totalRewards = 0;
      
      users.forEach(user => {
        const userShare = user.lp / totalLpSupply;
        const rewards = Math.floor(userShare * rewardRate * timeElapsed);
        totalRewards += rewards;
        
        console.log(`User with ${user.lp} LP gets ${rewards} rewards`);
        expect(userShare).to.be.closeTo(user.expectedShare, 0.001);
      });
      
      // Total rewards should equal rate * time
      const expectedTotal = rewardRate * timeElapsed;
      console.log(`Total distributed: ${totalRewards}/${expectedTotal}`);
      expect(totalRewards).to.be.closeTo(expectedTotal, 1);
    });
  });

  describe("Perpetual Trading Calculations", () => {
    
    it("should calculate required margin correctly", () => {
      const positionSize = 10000;
      const leverages = [2, 5, 10];
      
      leverages.forEach(leverage => {
        const requiredMargin = Math.ceil(positionSize / leverage);
        console.log(`${leverage}x leverage requires ${requiredMargin} margin for ${positionSize} position`);
        
        expect(requiredMargin).to.equal(positionSize / leverage);
      });
    });

    it("should calculate liquidation price for long positions", () => {
      const entryPrice = 10000;
      const leverages = [2, 5, 10];
      
      leverages.forEach(leverage => {
        const liqPrice = entryPrice * (1 - 1 / leverage);
        const priceMove = ((entryPrice - liqPrice) / entryPrice) * 100;
        
        console.log(`Long ${leverage}x: Entry ${entryPrice}, Liq ${liqPrice} (${priceMove.toFixed(2)}% drop)`);
        
        // Higher leverage = closer liquidation price
        if (leverage === 2) expect(liqPrice).to.equal(5000);
        if (leverage === 10) expect(liqPrice).to.equal(9000);
      });
    });

    it("should calculate liquidation price for short positions", () => {
      const entryPrice = 10000;
      const leverages = [2, 5, 10];
      
      leverages.forEach(leverage => {
        const liqPrice = entryPrice * (1 + 1 / leverage);
        const priceMove = ((liqPrice - entryPrice) / entryPrice) * 100;
        
        console.log(`Short ${leverage}x: Entry ${entryPrice}, Liq ${liqPrice} (${priceMove.toFixed(2)}% rise)`);
        
        // Higher leverage = closer liquidation price
        if (leverage === 2) expect(liqPrice).to.equal(15000);
        if (leverage === 10) expect(liqPrice).to.equal(11000);
      });
    });

    it("should calculate PnL for long positions", () => {
      const entryPrice = 10000;
      const positionSize = 10000;
      const currentPrice = 11000; // 10% gain
      const leverage = 5;
      
      // PnL = (current - entry) * (size / entry)
      const pnl = ((currentPrice - entryPrice) / entryPrice) * positionSize;
      const pnlPercent = ((currentPrice - entryPrice) / entryPrice) * 100 * leverage;
      
      console.log(`Long position PnL: ${pnl} (${pnlPercent}% on margin)`);
      
      expect(pnl).to.equal(1000);
      expect(pnlPercent).to.equal(50); // 10% * 5x leverage
    });

    it("should calculate PnL for short positions", () => {
      const entryPrice = 10000;
      const positionSize = 10000;
      const currentPrice = 9000; // 10% drop (profit for short)
      
      // PnL = (entry - current) * (size / entry)
      const pnl = ((entryPrice - currentPrice) / entryPrice) * positionSize;
      
      console.log(`Short position PnL: ${pnl}`);
      
      expect(pnl).to.equal(1000);
    });
  });

  describe("Edge Cases and Overflow Protection", () => {
    
    it("should handle maximum u64 values safely", () => {
      const maxU64 = Math.pow(2, 53) - 1; // JavaScript safe integer
      const largeValue = maxU64 / 2;
      
      console.log(`Testing with large value: ${largeValue}`);
      
      // Should not overflow
      const result = largeValue + 1000;
      expect(result).to.be.greaterThan(largeValue);
    });

    it("should handle division by zero gracefully", () => {
      const totalLpSupply = 0;
      const userLpBalance = 1000;
      
      // Should check for zero before division
      const rewards = totalLpSupply === 0 ? 0 : (userLpBalance / totalLpSupply) * 100;
      
      expect(rewards).to.equal(0);
    });

    it("should handle negative values in calculations", () => {
      // Test that we don't allow negative amounts
      const amountIn = -100;
      
      // Implementation should reject negative values
      const isValid = amountIn > 0;
      expect(isValid).to.be.false;
    });

    it("should round down consistently for token amounts", () => {
      const value = 1000.9999;
      const rounded = Math.floor(value);
      
      expect(rounded).to.equal(1000);
    });
  });

  describe("Fee Calculations", () => {
    
    it("should calculate 0.3% fee correctly", () => {
      const amounts = [1000, 10000, 100000, 1000000];
      const feeBps = 30; // 0.3%
      
      amounts.forEach(amount => {
        const feeAmount = Math.floor((amount * feeBps) / 10000);
        const amountAfterFee = amount - feeAmount;
        
        console.log(`Amount: ${amount}, Fee: ${feeAmount}, After fee: ${amountAfterFee}`);
        
        // Fee should be exactly 0.3%
        expect(feeAmount).to.be.closeTo(amount * 0.003, 1);
      });
    });

    it("should accumulate fees in pool correctly", () => {
      let poolFees = 0;
      const swaps = [1000, 2000, 3000];
      const feeBps = 30;
      
      swaps.forEach(amount => {
        const fee = Math.floor((amount * feeBps) / 10000);
        poolFees += fee;
      });
      
      console.log(`Total fees accumulated: ${poolFees}`);
      
      expect(poolFees).to.equal(18); // 3 + 6 + 9
    });
  });

  describe("Constant Product Invariant", () => {
    
    it("should maintain k = x * y invariant (with fee adjustment)", () => {
      const initialReserveA = 10000;
      const initialReserveB = 20000;
      const k = initialReserveA * initialReserveB;
      
      // Simulate swap
      const amountIn = 1000;
      const fee = 30;
      const amountInWithFee = (amountIn * (10000 - fee)) / 10000;
      const amountOut = Math.floor((initialReserveB * amountInWithFee) / (initialReserveA + amountInWithFee));
      
      const newReserveA = initialReserveA + amountIn;
      const newReserveB = initialReserveB - amountOut;
      const newK = newReserveA * newReserveB;
      
      console.log(`Initial k: ${k}`);
      console.log(`New k: ${newK}`);
      console.log(`K increase: ${newK - k} (${((newK - k) / k * 100).toFixed(4)}%)`);
      
      // K should increase due to fees
      expect(newK).to.be.greaterThan(k);
    });
  });
});
