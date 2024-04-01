use crate::*;

#[account]
pub struct Vault {
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub total_lp: u64,
    pub bump: u8,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub lp: u64,
    pub staking_end: i64,

    pub initialized: bool,
    pub bump: u8,
}



