use anchor_lang::prelude::*;

/// Admin state
#[account]
pub struct AdminState {
    pub bump: u8,
    pub admin: Pubkey,
    pub is_paused: bool,
    pub pause_reason: String,
    pub last_action_time: i64,
    pub fee_update_count: u64,
}

impl AdminState {
    pub const LEN: usize = 8 + 1 + 32 + 1 + (4 + 200) + 8 + 8;
}
