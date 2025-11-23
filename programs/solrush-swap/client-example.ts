/**
 * SolRush Swap Client Examples
 * 
 * Demonstrates how to use all order types:
 * - Market Orders
 * - Limit Orders  
 * - DCA Orders
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { SolrushSwap } from "../target/types/solrush_swap";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";

// Trading pair enum helper
const TradingPair = {
  SolUsdc: { solUsdc: {} },
  SolWeth: { solWeth: {} },
  SolUsdt: { solUsdt: {} },
};

const OrderSide = {
  Buy: { buy: {} },
  Sell: { sell: {} },
};

export class SolRushSwapClient {
  constructor(
    public program: Program<SolrushSwap>,
    public provider: anchor.AnchorProvider
  ) {}

  /**
   * Get pool PDA for a trading pair
   */
  getPoolPda(tradingPair: any): [PublicKey, number] {
    const tradingPairBuffer = Buffer.from(
      this.serializeTradingPair(tradingPair)
    );
    return PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tradingPairBuffer],
      this.program.programId
    );
  }

  /**
   * Get order book PDA
   */
  getOrderBookPda(tradingPair: any): [PublicKey, number] {
    const tradingPairBuffer = Buffer.from(
      this.serializeTradingPair(tradingPair)
    );
    return PublicKey.findProgramAddressSync(
      [Buffer.from("order_book"), tradingPairBuffer],
      this.program.programId
    );
  }

  /**
   * Initialize a new liquidity pool
   */
  async initializePool(
    tradingPair: any,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    feeRate: number = 30 // 0.3%
  ) {
    const [poolPda] = this.getPoolPda(tradingPair);

    const tokenAVault = Keypair.generate();
    const tokenBVault = Keypair.generate();
    const lpTokenMint = Keypair.generate();

    const tx = await this.program.methods
      .initializePool(tradingPair, feeRate)
      .accounts({
        authority: this.provider.wallet.publicKey,
        pool: poolPda,
        tokenAMint,
        tokenBMint,
        tokenAVault: tokenAVault.publicKey,
        tokenBVault: tokenBVault.publicKey,
        lpTokenMint: lpTokenMint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([tokenAVault, tokenBVault, lpTokenMint])
      .rpc();

    console.log("Pool initialized:", tx);
    return { poolPda, tx };
  }

  /**
   * Initialize order book
   */
  async initializeOrderBook(tradingPair: any) {
    const [poolPda] = this.getPoolPda(tradingPair);
    const [orderBookPda] = this.getOrderBookPda(tradingPair);

    const tx = await this.program.methods
      .initializeOrderBook(tradingPair)
      .accounts({
        authority: this.provider.wallet.publicKey,
        pool: poolPda,
        orderBook: orderBookPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Order book initialized:", tx);
    return { orderBookPda, tx };
  }

  /**
   * Execute a market order
   * 
   * @example
   * // Buy 100 USDC with SOL, max 1% slippage
   * await client.executeMarketOrder(
   *   TradingPair.SolUsdc,
   *   OrderSide.Buy,
   *   new BN(1_000_000_000), // 1 SOL
   *   new BN(99_000_000),     // Min 99 USDC
   *   100                      // 1% slippage
   * );
   */
  async executeMarketOrder(
    tradingPair: any,
    orderSide: any,
    amountIn: BN,
    minimumAmountOut: BN,
    slippageTolerance: number
  ) {
    const [poolPda] = this.getPoolPda(tradingPair);
    const pool = await this.program.account.liquidityPool.fetch(poolPda);

    // Get user token accounts (must be created beforehand)
    const userTokenIn = await this.getUserTokenAccount(
      orderSide === OrderSide.Buy ? pool.tokenAMint : pool.tokenBMint
    );
    const userTokenOut = await this.getUserTokenAccount(
      orderSide === OrderSide.Buy ? pool.tokenBMint : pool.tokenAMint
    );

    // Generate PDA for order record
    const timestamp = Math.floor(Date.now() / 1000);
    const [orderRecordPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("market_order"),
        this.provider.wallet.publicKey.toBuffer(),
        Buffer.from(new BN(timestamp).toArray("le", 8)),
      ],
      this.program.programId
    );

    const tx = await this.program.methods
      .executeMarketOrder(amountIn, minimumAmountOut, slippageTolerance, orderSide)
      .accounts({
        user: this.provider.wallet.publicKey,
        pool: poolPda,
        userTokenIn,
        userTokenOut,
        poolTokenAVault: pool.tokenAVault,
        poolTokenBVault: pool.tokenBVault,
        orderRecord: orderRecordPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Market order executed:", tx);
    return { tx, orderRecordPda };
  }

  /**
   * Place a limit order
   * 
   * @example
   * // Buy USDC when price reaches 150 or lower
   * await client.placeLimitOrder(
   *   TradingPair.SolUsdc,
   *   OrderSide.Buy,
   *   new BN(1_000_000_000),   // 1 SOL
   *   new BN(150_000_000_000), // 150 USDC/SOL
   *   100,                      // 1% slippage
   *   0                         // Never expires
   * );
   */
  async placeLimitOrder(
    tradingPair: any,
    orderSide: any,
    amountIn: BN,
    limitPrice: BN,
    slippageTolerance: number,
    expiresAt: number = 0
  ) {
    const [poolPda] = this.getPoolPda(tradingPair);
    const [orderBookPda] = this.getOrderBookPda(tradingPair);
    const pool = await this.program.account.liquidityPool.fetch(poolPda);
    const orderBook = await this.program.account.orderBook.fetch(orderBookPda);

    const orderIndex =
      orderBook.buyOrdersCount.toNumber() + orderBook.sellOrdersCount.toNumber();

    const [limitOrderPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("limit_order"),
        this.provider.wallet.publicKey.toBuffer(),
        orderBookPda.toBuffer(),
        Buffer.from(new BN(orderIndex).toArray("le", 8)),
      ],
      this.program.programId
    );

    const userTokenAccount = await this.getUserTokenAccount(
      orderSide === OrderSide.Buy ? pool.tokenAMint : pool.tokenBMint
    );

    const escrowTokenAccount = Keypair.generate();

    const tx = await this.program.methods
      .placeLimitOrder(
        tradingPair,
        orderSide,
        amountIn,
        limitPrice,
        slippageTolerance,
        new BN(expiresAt)
      )
      .accounts({
        user: this.provider.wallet.publicKey,
        pool: poolPda,
        orderBook: orderBookPda,
        limitOrder: limitOrderPda,
        userTokenAccount,
        escrowTokenAccount: escrowTokenAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([escrowTokenAccount])
      .rpc();

    console.log("Limit order placed:", tx);
    return { tx, limitOrderPda };
  }

  /**
   * Cancel a limit order
   */
  async cancelLimitOrder(limitOrderPda: PublicKey) {
    const limitOrder = await this.program.account.limitOrder.fetch(
      limitOrderPda
    );
    const [orderBookPda] = this.getOrderBookPda(limitOrder.tradingPair);

    const tx = await this.program.methods
      .cancelLimitOrder()
      .accounts({
        user: this.provider.wallet.publicKey,
        orderBook: orderBookPda,
        limitOrder: limitOrderPda,
        userTokenAccount: limitOrder.userTokenAccount,
        escrowTokenAccount: limitOrder.escrowTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Limit order cancelled:", tx);
    return tx;
  }

  /**
   * Execute a limit order (keeper bot function)
   */
  async executeLimitOrder(limitOrderPda: PublicKey) {
    const limitOrder = await this.program.account.limitOrder.fetch(
      limitOrderPda
    );
    const [poolPda] = this.getPoolPda(limitOrder.tradingPair);
    const [orderBookPda] = this.getOrderBookPda(limitOrder.tradingPair);
    const pool = await this.program.account.liquidityPool.fetch(poolPda);

    const tx = await this.program.methods
      .executeLimitOrder()
      .accounts({
        executor: this.provider.wallet.publicKey,
        pool: poolPda,
        orderBook: orderBookPda,
        limitOrder: limitOrderPda,
        orderOwner: limitOrder.owner,
        orderOwnerTokenAccount: limitOrder.userTokenAccount,
        escrowTokenAccount: limitOrder.escrowTokenAccount,
        poolTokenAVault: pool.tokenAVault,
        poolTokenBVault: pool.tokenBVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Limit order executed:", tx);
    return tx;
  }

  /**
   * Create a DCA order
   * 
   * @example
   * // DCA: Buy $100 USDC every day for 30 days
   * await client.createDCAOrder(
   *   TradingPair.SolUsdc,
   *   OrderSide.Buy,
   *   new BN(100_000_000),      // 100 USDC per cycle
   *   30,                        // 30 cycles
   *   86400,                     // 24 hours
   *   200,                       // 2% slippage
   *   new BN(140_000_000_000),  // Min price 140
   *   new BN(160_000_000_000)   // Max price 160
   * );
   */
  async createDCAOrder(
    tradingPair: any,
    orderSide: any,
    amountPerCycle: BN,
    totalCycles: number,
    cycleFrequency: number,
    slippageTolerance: number,
    minPrice: BN = new BN(0),
    maxPrice: BN = new BN(0)
  ) {
    const [poolPda] = this.getPoolPda(tradingPair);
    const pool = await this.program.account.liquidityPool.fetch(poolPda);

    const timestamp = Math.floor(Date.now() / 1000);
    const [dcaOrderPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("dca_order"),
        this.provider.wallet.publicKey.toBuffer(),
        Buffer.from(new BN(timestamp).toArray("le", 8)),
      ],
      this.program.programId
    );

    const userTokenAccount = await this.getUserTokenAccount(
      orderSide === OrderSide.Buy ? pool.tokenAMint : pool.tokenBMint
    );

    const escrowTokenAccount = Keypair.generate();

    const tx = await this.program.methods
      .createDcaOrder(
        tradingPair,
        orderSide,
        amountPerCycle,
        totalCycles,
        new BN(cycleFrequency),
        slippageTolerance,
        minPrice,
        maxPrice
      )
      .accounts({
        user: this.provider.wallet.publicKey,
        pool: poolPda,
        dcaOrder: dcaOrderPda,
        userTokenAccount,
        escrowTokenAccount: escrowTokenAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([escrowTokenAccount])
      .rpc();

    console.log("DCA order created:", tx);
    return { tx, dcaOrderPda };
  }

  /**
   * Execute a DCA order cycle (keeper bot function)
   */
  async executeDCAOrder(dcaOrderPda: PublicKey) {
    const dcaOrder = await this.program.account.dcaOrder.fetch(dcaOrderPda);
    const [poolPda] = this.getPoolPda(dcaOrder.tradingPair);
    const pool = await this.program.account.liquidityPool.fetch(poolPda);

    const tx = await this.program.methods
      .executeDcaOrder()
      .accounts({
        executor: this.provider.wallet.publicKey,
        pool: poolPda,
        dcaOrder: dcaOrderPda,
        orderOwner: dcaOrder.owner,
        orderOwnerTokenAccount: dcaOrder.userTokenAccount,
        escrowTokenAccount: dcaOrder.escrowTokenAccount,
        poolTokenAVault: pool.tokenAVault,
        poolTokenBVault: pool.tokenBVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("DCA order cycle executed:", tx);
    return tx;
  }

  /**
   * Cancel a DCA order
   */
  async cancelDCAOrder(dcaOrderPda: PublicKey) {
    const dcaOrder = await this.program.account.dcaOrder.fetch(dcaOrderPda);

    const tx = await this.program.methods
      .cancelDcaOrder()
      .accounts({
        user: this.provider.wallet.publicKey,
        dcaOrder: dcaOrderPda,
        userTokenAccount: dcaOrder.userTokenAccount,
        escrowTokenAccount: dcaOrder.escrowTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("DCA order cancelled:", tx);
    return tx;
  }

  /**
   * Get current market price from pool
   */
  async getMarketPrice(tradingPair: any): Promise<number> {
    const [poolPda] = this.getPoolPda(tradingPair);
    const pool = await this.program.account.liquidityPool.fetch(poolPda);

    // Price in token B per token A
    const price =
      (pool.reserveB.toNumber() * 1e9) / pool.reserveA.toNumber();
    return price;
  }

  /**
   * Get all open limit orders for a user
   */
  async getUserLimitOrders(userPubkey: PublicKey) {
    const orders = await this.program.account.limitOrder.all([
      {
        memcmp: {
          offset: 8, // After discriminator
          bytes: userPubkey.toBase58(),
        },
      },
    ]);

    return orders.filter(
      (order) =>
        order.account.orderStatus.open || order.account.orderStatus.partiallyFilled
    );
  }

  /**
   * Get all active DCA orders for a user
   */
  async getUserDCAOrders(userPubkey: PublicKey) {
    const orders = await this.program.account.dcaOrder.all([
      {
        memcmp: {
          offset: 8, // After discriminator
          bytes: userPubkey.toBase58(),
        },
      },
    ]);

    return orders.filter(
      (order) =>
        order.account.orderStatus.open || order.account.orderStatus.partiallyFilled
    );
  }

  // Helper methods
  private async getUserTokenAccount(mint: PublicKey): Promise<PublicKey> {
    // Implement token account lookup/creation
    // This is a placeholder - implement based on your token account strategy
    throw new Error("Implement getUserTokenAccount");
  }

  private serializeTradingPair(tradingPair: any): Uint8Array {
    // Serialize trading pair enum
    if (tradingPair.solUsdc) return new Uint8Array([0]);
    if (tradingPair.solWeth) return new Uint8Array([1]);
    if (tradingPair.solUsdt) return new Uint8Array([2]);
    throw new Error("Invalid trading pair");
  }
}

// Example usage
async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolrushSwap as Program<SolrushSwap>;
  const client = new SolRushSwapClient(program, provider);

  // Example 1: Market Order
  console.log("\n=== Market Order Example ===");
  await client.executeMarketOrder(
    TradingPair.SolUsdc,
    OrderSide.Buy,
    new BN(1_000_000_000), // 1 SOL
    new BN(145_000_000),   // Min 145 USDC
    100                     // 1% slippage
  );

  // Example 2: Limit Order
  console.log("\n=== Limit Order Example ===");
  const { limitOrderPda } = await client.placeLimitOrder(
    TradingPair.SolUsdc,
    OrderSide.Buy,
    new BN(1_000_000_000),
    new BN(150_000_000_000),
    100,
    0
  );

  // Example 3: DCA Order
  console.log("\n=== DCA Order Example ===");
  await client.createDCAOrder(
    TradingPair.SolUsdc,
    OrderSide.Buy,
    new BN(100_000_000),
    30,
    86400,
    200,
    new BN(140_000_000_000),
    new BN(160_000_000_000)
  );
}

// Run if executed directly
if (require.main === module) {
  main().catch(console.error);
}

export { TradingPair, OrderSide };
