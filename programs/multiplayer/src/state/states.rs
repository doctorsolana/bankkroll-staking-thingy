use crate::*;


#[account]
pub struct Game {
    pub game_maker: Pubkey,
    pub state: GameState,
    pub mint: Pubkey,
    pub max_players: u32,
    pub players: Vec<Player>,
    pub winners: u8,
    pub game_id: u64,
    pub unix_timestamp_str: String,
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
