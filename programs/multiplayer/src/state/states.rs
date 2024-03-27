use crate::*;


#[account]
pub struct GambaState {
    pub rng: Pubkey,
    pub authority: Pubkey,
    pub gamba_fee_bps: u32,
    pub initialized: bool,
    pub bump: u8,
}

#[account]
pub struct Game {
    pub game_maker: Pubkey,
    pub state: GameState,
    pub mint: Pubkey,
    pub max_players: u8,
    pub players: Vec<Player>,
    pub winners: u8,
    pub game_id: u64,
    pub game_expiration_timestamp: i64,
    pub unique_identifier: u32,
    pub wager_type: WagerType,
    pub wager: u64,
    pub bump: u8,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Player {
    pub creator_address_ata: Pubkey,
    pub user_ata: Pubkey,
    pub creator_fee_amount: u64, // Actual fee amount in terms of the wager
    pub gamba_fee_amount: u64, // 
    pub wager: u64,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Waiting,
    Playing,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum WagerType {
    SameWager,
    CustomWager,
}
