use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

declare_id!("EC6VgGSamTvY3XRuYC7uyW1DZMrvuRkfztdy6YeCfNxX");

#[program]
mod hello_anchor {
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, max_players: u32) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        game_account.game_maker = *ctx.accounts.game_maker.key;
        game_account.state = GameState::Waiting;
        game_account.max_players = max_players;
        game_account.players = Vec::new();
        game_account.game_id = 0;
        game_account.mint = *ctx.accounts.mint.to_account_info().key;
        Ok(())
    }

    pub fn join_game(
        ctx: Context<JoinLeaveGame>,
        creator_fee: u32,
        wager: u64,
        creator_address: Pubkey,
    ) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        let player_account = &ctx.accounts.player_account;

        // calcualte creator fee amount from creator fee bip
        let creator_fee_amount = (wager * creator_fee as u64) / 10000;

        let player = Player {
            creator_address: creator_address,
            user: *player_account.key,
            creator_fee_amount: creator_fee_amount,
            wager: wager,
        };

        //check that the game is in waiting state
        if game_account.state != GameState::Waiting {
            return Err(ErrorCode::PlayerAlreadyInGame.into());
        };

        //check that the player is not already in the game
        for p in game_account.players.iter() {
            if p.user == player.user {
                return Err(ErrorCode::PlayerAlreadyInGame.into());
            }
        }
        //add the player to the game
        game_account.players.push(player);

        //if max players is reached, start the game
        if game_account.players.len() == game_account.max_players as usize {
            game_account.state = GameState::Playing;
        }
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(max_players: u32)] // just testing
pub struct CreateGame<'info> {
    #[account(init, payer = game_maker, space = 8 + 1000, seeds = [b"GAME",game_maker.key().as_ref(), &max_players.to_le_bytes()], bump )] // Adjust space as needed
    pub game_account: Account<'info, Game>,
    //mint account
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = game_maker,
        token::mint = mint,
        token::authority = game_account, 
        seeds = [game_account.key().as_ref()], bump
          )]
    pub game_account_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub game_maker: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    
}

#[derive(Accounts)]
pub struct JoinLeaveGame<'info> {
    #[account(mut)]
    pub game_account: Account<'info, Game>,

    #[account(mut)]
    pub player_account: Signer<'info>,

    //mint account
    #[account(address = game_account.mint)]
    pub mint: Account<'info, Mint>,

    //game associated token account
    #[account(mut, seeds = [game_account.key().as_ref()], bump)]
    pub game_account_token: Account<'info, TokenAccount>,

    //player associated token account
    #[account(mut, 
    associated_token::mint = mint,
    associated_token::authority = player_account,)]
    pub player_account_token: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}


#[account]
pub struct Game {
    pub game_maker: Pubkey,
    pub state: GameState,
    pub mint: Pubkey,
    pub max_players: u32,
    pub players: Vec<Player>,
    pub game_id: u64,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Player {
    creator_address: Pubkey,
    user: Pubkey,
    creator_fee_amount: u64, // Actual fee amount in terms of the wager
    wager: u64,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Waiting,
    Playing,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Player is already in the game")]
    PlayerAlreadyInGame,
}
