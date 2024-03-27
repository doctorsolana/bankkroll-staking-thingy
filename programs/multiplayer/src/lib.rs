use anchor_lang::prelude::*;

// TODO
// - Add 2 different game modes, custom wager and all same wager,
// - Add 1 winner or multiple winners

declare_id!("2bH8ug93GjpG7SpLmb4u1Pb67CHtWUuxBbauEsp4gWkJ");

pub mod error;
pub use error::*;

pub mod events;
pub use events::*;

mod instructions;
mod state;

pub use state::*;
pub use instructions::*;

#[program]
mod multiplayer {
    use super::*;

    pub fn create_game(
        ctx: Context<CreateGame>,
        max_players: u8,
        winners: u8,
        duration_seconds: i64,
        unique_identifier: u32,
        wager_type: u8, 
        wager: u64,
    ) -> Result<()> {
        instructions::create_game_handler(
            ctx,
            max_players,
            winners,
            duration_seconds,
            unique_identifier,
            wager_type,
            wager,
        )
    }

    pub fn join_game(ctx: Context<JoinGame>, creator_fee: u32, wager: u64) -> Result<()> {
        instructions::join_game_handler(ctx, creator_fee, wager)
    }

    pub fn leave_game(ctx: Context<LeaveGame>) -> Result<()> {
        instructions::leave_game_handler(ctx)
    }

    pub fn settle_game(ctx: Context<SettleGame>) -> Result<()> {
        instructions::settle_game_handler(ctx)
    }

    pub fn gamba_config(ctx: Context<GambaConfig>, gamba_fee: u32, rng: Pubkey) -> Result<()> {
        instructions::gamba_config_handler(ctx, gamba_fee, rng)
    }
}
