use anchor_lang::prelude::*;

declare_id!("SRAdmin111111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

#[program]
pub mod solrush_admin {
    use super::*;

    /// Initialize admin authority
    pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
        instructions::initialize_admin::handler(ctx)
    }

    /// Pause all trading operations
    pub fn pause_trading(ctx: Context<PauseTrading>, reason: String) -> Result<()> {
        instructions::pause_trading::handler(ctx, reason)
    }

    /// Resume trading operations
    pub fn resume_trading(ctx: Context<ResumeTrading>) -> Result<()> {
        instructions::resume_trading::handler(ctx)
    }

    /// Update fee rate for a pool
    pub fn update_fee_rate(ctx: Context<UpdateFeeRate>, new_rate: u16) -> Result<()> {
        instructions::update_fee_rate::handler(ctx, new_rate)
    }

    /// Transfer admin authority
    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::transfer_admin::handler(ctx, new_admin)
    }

    /// Emergency withdraw from pool (when paused)
    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        instructions::emergency_withdraw::handler(ctx)
    }
}
