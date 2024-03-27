use anchor_lang::prelude::*;

#[error_code]
pub enum GambaError {
    #[msg("Player is already in the game")]
    PlayerAlreadyInGame,
    #[msg("Player is not in the game")]
    PlayerNotInGame,
    #[msg("Game is already in progress")]
    GameInProgress,
    #[msg("Invalid game account")]
    InvalidGameAccount,
    #[msg("Cannot settle yet")]
    CannotSettleYet,
    #[msg("Authority mismatch")]
    AuthorityMismatch,

}