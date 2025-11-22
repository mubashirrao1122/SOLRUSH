use anchor_lang::prelude::*;

declare_id!("SRLPooL111111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod solrush_liquidity_pool {
    use super::*;

    /// Initialize a new liquidity pool for a token pair
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `fee_rate` - Trading fee rate in basis points (e.g., 30 = 0.3%)
    pub fn initialize_pool(ctx: Context<InitializePool>, fee_rate: u16) -> Result<()> {
        instructions::initialize_pool::handler(ctx, fee_rate)
    }

    /// Add liquidity to an existing pool
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `token_a_amount` - Amount of token A to deposit
    /// * `token_b_amount` - Amount of token B to deposit
    /// * `min_lp_tokens` - Minimum LP tokens to receive (slippage protection)
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        token_a_amount: u64,
        token_b_amount: u64,
        min_lp_tokens: u64,
    ) -> Result<()> {
        instructions::add_liquidity::handler(ctx, token_a_amount, token_b_amount, min_lp_tokens)
    }

    /// Remove liquidity from a pool
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `lp_token_amount` - Amount of LP tokens to burn
    /// * `min_token_a` - Minimum token A to receive
    /// * `min_token_b` - Minimum token B to receive
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_token_amount: u64,
        min_token_a: u64,
        min_token_b: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, lp_token_amount, min_token_a, min_token_b)
    }

    /// Swap tokens in the pool
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `amount_in` - Amount of input token
    /// * `minimum_amount_out` - Minimum output amount (slippage protection)
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, minimum_amount_out)
    }

    /// Get pool information
    pub fn get_pool_info(ctx: Context<GetPoolInfo>) -> Result<PoolInfo> {
        instructions::get_pool_info::handler(ctx)
    }
}
