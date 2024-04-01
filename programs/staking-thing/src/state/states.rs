use crate::*;


#[account]
pub struct Vault {
    pub mint: Pubkey,
    pub mint_token_account: Pubkey,
    pub bump: u8,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub lp: u64,
    pub staking_end: i64,
    pub bump: u8,
}



