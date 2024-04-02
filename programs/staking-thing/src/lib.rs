use anchor_lang::prelude::*;

declare_id!("4K2j5o3py74iRKPKTtS3NgqQUhGwV1a3aLZFZQFwYxGD");

pub mod error;
pub use error::*;

pub mod events;
pub use events::*;

mod instructions;
pub use instructions::*;

mod state;
pub use state::*;

#[program]
mod staking_thing {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        instructions::create_vault_handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit_handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw_handler(ctx, amount)
    }
    
}
