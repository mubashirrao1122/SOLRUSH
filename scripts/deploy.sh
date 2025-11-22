#!/bin/bash

# SolRush DEX Deployment Script for Devnet
# This script deploys all SolRush programs to Solana Devnet

set -e

echo "🚀 Starting SolRush DEX Deployment to Devnet"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

if ! command -v solana &> /dev/null; then
    echo -e "${RED}Error: Solana CLI not found${NC}"
    exit 1
fi

if ! command -v anchor &> /dev/null; then
    echo -e "${RED}Error: Anchor CLI not found${NC}"
    exit 1
fi

# Check Solana config
CLUSTER=$(solana config get | grep "RPC URL" | awk '{print $3}')
echo "Current cluster: $CLUSTER"

if [[ ! "$CLUSTER" == *"devnet"* ]]; then
    echo -e "${YELLOW}Switching to devnet...${NC}"
    solana config set --url devnet
fi

# Check wallet balance
BALANCE=$(solana balance | awk '{print $1}')
echo "Wallet balance: $BALANCE SOL"

if (( $(echo "$BALANCE < 5" | bc -l) )); then
    echo -e "${YELLOW}Low balance. Requesting airdrop...${NC}"
    solana airdrop 2
    sleep 5
fi

# Build programs
echo -e "\n${YELLOW}Building programs...${NC}"
anchor build

# Generate program IDs
echo -e "\n${YELLOW}Generating program IDs...${NC}"
anchor keys list

# Deploy programs
echo -e "\n${GREEN}Deploying programs to devnet...${NC}"

echo "Deploying Liquidity Pool..."
anchor deploy --program-name solrush_liquidity_pool --provider.cluster devnet

echo "Deploying Swap..."
anchor deploy --program-name solrush_swap --provider.cluster devnet

echo "Deploying Rush Token..."
anchor deploy --program-name solrush_token --provider.cluster devnet

echo "Deploying Rewards..."
anchor deploy --program-name solrush_rewards --provider.cluster devnet

echo "Deploying Perpetual..."
anchor deploy --program-name solrush_perpetual --provider.cluster devnet

echo "Deploying Admin..."
anchor deploy --program-name solrush_admin --provider.cluster devnet

echo -e "\n${GREEN}✅ All programs deployed successfully!${NC}"

# Display program IDs
echo -e "\n${YELLOW}Program IDs:${NC}"
anchor keys list

echo -e "\n${GREEN}Deployment complete!${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Update program IDs in Anchor.toml"
echo "2. Run initialization script: ./scripts/initialize.sh"
echo "3. Run tests: anchor test --skip-deploy"

exit 0
