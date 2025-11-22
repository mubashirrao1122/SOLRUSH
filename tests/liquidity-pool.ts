import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  createMint, 
  getOrCreateAssociatedTokenAccount,
  mintTo 
} from "@solana/spl-token";
import { assert } from "chai";
import { SolrushLiquidityPool } from "../target/types/solrush_liquidity_pool";

describe("solrush-liquidity-pool", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolrushLiquidityPool as Program<SolrushLiquidityPool>;

  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
  let poolPda: PublicKey;
  let authorityPda: PublicKey;
  let lpTokenMintPda: PublicKey;
  let tokenAVault: PublicKey;
  let tokenBVault: PublicKey;
  let userTokenA: any;
  let userTokenB: any;
  let userLpToken: any;

  before(async () => {
    // Create token mints
    const payer = (provider.wallet as any).payer;
    
    tokenAMint = await createMint(
      provider.connection,
      payer,
      provider.wallet.publicKey,
      null,
      9
    );

    tokenBMint = await createMint(
      provider.connection,
      payer,
      provider.wallet.publicKey,
      null,
      9
    );

    // Create user token accounts
    userTokenA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      tokenAMint,
      provider.wallet.publicKey
    );

    userTokenB = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      tokenBMint,
      provider.wallet.publicKey
    );

    // Mint tokens to user
    await mintTo(
      provider.connection,
      payer,
      tokenAMint,
      userTokenA.address,
      provider.wallet.publicKey,
      1000000000000
    );

    await mintTo(
      provider.connection,
      payer,
      tokenBMint,
      userTokenB.address,
      provider.wallet.publicKey,
      1000000000000
    );

    // Derive PDAs
    [poolPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer(),
      ],
      program.programId
    );

    [authorityPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool_authority"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer(),
      ],
      program.programId
    );

    [lpTokenMintPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("lp_token"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer(),
      ],
      program.programId
    );
  });

  it("Initializes a liquidity pool", async () => {
    const tokenAVaultKeypair = Keypair.generate();
    const tokenBVaultKeypair = Keypair.generate();

    tokenAVault = tokenAVaultKeypair.publicKey;
    tokenBVault = tokenBVaultKeypair.publicKey;

    const feeRate = 30; // 0.3%

    const tx = await program.methods
      .initializePool(feeRate)
      .accounts({
        pool: poolPda,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        tokenAVault: tokenAVault,
        tokenBVault: tokenBVault,
        lpTokenMint: lpTokenMintPda,
        poolAuthority: authorityPda,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([tokenAVaultKeypair, tokenBVaultKeypair])
      .rpc();

    console.log("Initialize pool transaction:", tx);

    const poolAccount = await program.account.poolState.fetch(poolPda);
    assert.equal(poolAccount.feeRate, feeRate);
    assert.equal(poolAccount.tokenAReserve.toNumber(), 0);
    assert.equal(poolAccount.tokenBReserve.toNumber(), 0);
  });

  it("Adds liquidity to the pool", async () => {
    const payer = (provider.wallet as any).payer;

    userLpToken = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      lpTokenMintPda,
      provider.wallet.publicKey
    );

    const tokenAAmount = new anchor.BN(1000000000);
    const tokenBAmount = new anchor.BN(1000000000);
    const minLpTokens = new anchor.BN(900000000);

    const tx = await program.methods
      .addLiquidity(tokenAAmount, tokenBAmount, minLpTokens)
      .accounts({
        pool: poolPda,
        tokenAVault: tokenAVault,
        tokenBVault: tokenBVault,
        lpTokenMint: lpTokenMintPda,
        userTokenA: userTokenA.address,
        userTokenB: userTokenB.address,
        userLpToken: userLpToken.address,
        poolAuthority: authorityPda,
        user: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Add liquidity transaction:", tx);

    const poolAccount = await program.account.poolState.fetch(poolPda);
    assert.ok(poolAccount.tokenAReserve.toNumber() > 0);
    assert.ok(poolAccount.tokenBReserve.toNumber() > 0);
    assert.ok(poolAccount.lpTokenSupply.toNumber() > 0);
  });

  it("Executes a swap", async () => {
    const amountIn = new anchor.BN(1000000);
    const minimumAmountOut = new anchor.BN(990000);

    const userTokenABefore = await provider.connection.getTokenAccountBalance(userTokenA.address);

    const tx = await program.methods
      .swap(amountIn, minimumAmountOut)
      .accounts({
        pool: poolPda,
        tokenAVault: tokenAVault,
        tokenBVault: tokenBVault,
        userInputToken: userTokenA.address,
        userOutputToken: userTokenB.address,
        poolAuthority: authorityPda,
        user: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Swap transaction:", tx);

    const userTokenAAfter = await provider.connection.getTokenAccountBalance(userTokenA.address);
    assert.ok(Number(userTokenABefore.value.amount) > Number(userTokenAAfter.value.amount));
  });

  it("Removes liquidity from the pool", async () => {
    const lpTokenBalance = await provider.connection.getTokenAccountBalance(userLpToken.address);
    const lpTokenAmount = new anchor.BN(lpTokenBalance.value.amount).div(new anchor.BN(2));
    const minTokenA = new anchor.BN(1);
    const minTokenB = new anchor.BN(1);

    const tx = await program.methods
      .removeLiquidity(lpTokenAmount, minTokenA, minTokenB)
      .accounts({
        pool: poolPda,
        tokenAVault: tokenAVault,
        tokenBVault: tokenBVault,
        lpTokenMint: lpTokenMintPda,
        userTokenA: userTokenA.address,
        userTokenB: userTokenB.address,
        userLpToken: userLpToken.address,
        poolAuthority: authorityPda,
        user: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Remove liquidity transaction:", tx);

    const lpTokenBalanceAfter = await provider.connection.getTokenAccountBalance(userLpToken.address);
    assert.ok(Number(lpTokenBalanceAfter.value.amount) < Number(lpTokenBalance.value.amount));
  });

  it("Gets pool information", async () => {
    const poolInfo = await program.methods
      .getPoolInfo()
      .accounts({
        pool: poolPda,
      })
      .view();

    console.log("Pool Info:", poolInfo);
    assert.ok(poolInfo.tokenAReserve.toNumber() > 0);
    assert.ok(poolInfo.tokenBReserve.toNumber() > 0);
    assert.equal(poolInfo.feeRate, 30);
  });
});
