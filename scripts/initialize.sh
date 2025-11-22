#!/bin/bash

# SolRush DEX Initialization Script
# Initialize all programs after deployment

set -e

echo "🔧 Initializing SolRush DEX Programs"
echo "====================================="

# Use TypeScript/JavaScript for initialization
# This would typically call Anchor instructions

cat << 'EOF' > /tmp/initialize.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

async function initialize() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  console.log("Provider wallet:", provider.wallet.publicKey.toString());

  // 1. Initialize Admin
  console.log("\n1️⃣  Initializing Admin...");
  const adminProgram = anchor.workspace.SolrushAdmin;
  const [adminStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("admin_state")],
    adminProgram.programId
  );

  try {
    await adminProgram.methods
      .initializeAdmin()
      .accounts({
        adminState: adminStatePda,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("✅ Admin initialized");
  } catch (e) {
    console.log("⚠️  Admin may already be initialized");
  }

  // 2. Initialize Rush Token
  console.log("\n2️⃣  Initializing Rush Token...");
  const tokenProgram = anchor.workspace.SolrushToken;
  const [tokenStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("rush_token_state")],
    tokenProgram.programId
  );
  const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint_authority")],
    tokenProgram.programId
  );

  const tokenMint = Keypair.generate();

  try {
    await tokenProgram.methods
      .initializeRushToken()
      .accounts({
        tokenState: tokenStatePda,
        tokenMint: tokenMint.publicKey,
        mintAuthority: mintAuthorityPda,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([tokenMint])
      .rpc();
    console.log("✅ Rush Token initialized");
    console.log("   Token Mint:", tokenMint.publicKey.toString());
  } catch (e) {
    console.log("⚠️  Rush Token may already be initialized");
  }

  // 3. Initialize Rewards
  console.log("\n3️⃣  Initializing Rewards...");
  const rewardsProgram = anchor.workspace.SolrushRewards;
  const [rewardStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("reward_state")],
    rewardsProgram.programId
  );

  const rewardRatePerSecond = new anchor.BN(1000); // 1000 tokens per second

  try {
    await rewardsProgram.methods
      .initializeRewards(rewardRatePerSecond)
      .accounts({
        rewardState: rewardStatePda,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("✅ Rewards initialized");
    console.log("   Reward rate:", rewardRatePerSecond.toString(), "per second");
  } catch (e) {
    console.log("⚠️  Rewards may already be initialized");
  }

  console.log("\n✅ Initialization complete!");
  console.log("\nNext steps:");
  console.log("1. Create liquidity pools using initialize_pool");
  console.log("2. Add liquidity to pools");
  console.log("3. Start trading!");
}

initialize()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
EOF

echo "Running initialization..."
ts-node /tmp/initialize.ts

echo ""
echo "✅ Initialization complete!"

exit 0
