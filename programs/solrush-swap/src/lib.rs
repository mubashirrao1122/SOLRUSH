use anchor_lang::prelude::*;

declare_id!("SRSwap1111111111111111111111111111111111111");

pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod solrush_swap {
    use super::*;

    /// Execute a token swap with slippage protection
    /// 
    /// # Arguments
    /// * `ctx` - Context with required accounts
    /// * `amount_in` - Input token amount
    /// * `minimum_amount_out` - Minimum output amount (slippage protection)
    /// * `slippage_tolerance` - Slippage tolerance in basis points
    pub fn execute_swap(
        ctx: Context<ExecuteSwap>,
        amount_in: u64,
        minimum_amount_out: u64,
        slippage_tolerance: u16,
    ) -> Result<()> {
        instructions::execute_swap::handler(ctx, amount_in, minimum_amount_out, slippage_tolerance)
    }

    /// Calculate expected swap output
    /// 
    /// # Arguments
    /// * `ctx` - Context with pool account
    /// * `amount_in` - Input token amount
    /// * `input_is_token_a` - Whether input is token A
    pub fn calculate_swap_output(
        ctx: Context<CalculateSwapOutput>,
        amount_in: u64,
        input_is_token_a: bool,
    ) -> Result<SwapQuote> {
        instructions::calculate_swap_output::handler(ctx, amount_in, input_is_token_a)
    }

    /// Get swap fee information
    pub fn get_swap_fee_info(ctx: Context<GetSwapFeeInfo>) -> Result<SwapFeeInfo> {
        instructions::get_swap_fee_info::handler(ctx)
    }
}
