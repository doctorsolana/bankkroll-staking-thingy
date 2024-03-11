use anchor_lang::prelude::*;

// TODO
// - Add 2 different game modes, custom wager and all same wager,
// - Add 1 winner or multiple winners

declare_id!("6hGVGMWPQirQGq4j7KMo2r1t9YN9N7RjbRDkZ1Xu3a28");

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
        max_players: u32,
        winners: u8,
        unix_timestamp: String,
        wager_type: WagerType,
        wager: u64,
    ) -> Result<()> {
        instructions::create_game_handler(
            ctx,
            max_players,
            winners,
            unix_timestamp,
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
}
