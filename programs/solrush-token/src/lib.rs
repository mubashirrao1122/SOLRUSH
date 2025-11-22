use anchor_lang::prelude::*;

declare_id!("SRToken111111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

#[program]
pub mod solrush_token {
    use super::*;

    /// Initialize the RUSH token with supply cap
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    pub fn initialize_rush_token(ctx: Context<InitializeRushToken>) -> Result<()> {
        instructions::initialize_rush_token::handler(ctx)
    }

    /// Mint RUSH tokens (only by authorized minter)
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `amount` - Amount to mint (respecting cap)
    pub fn mint_rush_tokens(
        ctx: Context<MintRushTokens>,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_rush_tokens::handler(ctx, amount)
    }

    /// Transfer mint authority
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `new_authority` - New mint authority pubkey
    pub fn transfer_mint_authority(
        ctx: Context<TransferMintAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::transfer_mint_authority::handler(ctx, new_authority)
    }

    /// Get token supply information
    pub fn get_total_supply(ctx: Context<GetTotalSupply>) -> Result<SupplyInfo> {
        instructions::get_total_supply::handler(ctx)
    }

    /// Get circulating supply (total - locked)
    pub fn get_circulating_supply(ctx: Context<GetCirculatingSupply>) -> Result<u64> {
        instructions::get_circulating_supply::handler(ctx)
    }
}
