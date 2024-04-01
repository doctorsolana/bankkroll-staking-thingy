use anchor_lang::prelude::*;

declare_id!("5BYac1zRF6YKeovvcYrbi3vM1eDseDaMWiM6Lf4VF3ri");

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
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
        Ok(())
    }
    
}
