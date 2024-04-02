use anchor_lang::prelude::*;

declare_id!("2d9Tu5Z3i7dinTgsYmAhUkeLeb8qP3CZ3aY9sE7kxQDX");

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
