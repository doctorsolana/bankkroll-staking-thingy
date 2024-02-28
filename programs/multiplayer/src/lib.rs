use anchor_lang::prelude::*;

declare_id!("DgVtNY9ZPv68mmj69Hxo4pFTwo4RxfdCziy8irHj8dzp");

#[program]
pub mod multiplayer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
