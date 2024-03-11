use crate::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount,},
};

#[derive(Accounts)]
#[instruction(max_players: u32, unix_timestamp: String)] 
pub struct CreateGame<'info> {
    #[account(init, payer = game_maker, space = 8 + 1000, seeds = [b"GAME",game_maker.key().as_ref(), unix_timestamp.as_ref()], bump )] // use unix timestamp as seed
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
    pub game_account_ta_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub game_maker: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create_game_handler(ctx: Context<CreateGame>, max_players: u32, winners: u8, unix_timestamp: String, wager_type: WagerType, wager: u64 ) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.game_maker = *ctx.accounts.game_maker.key;
    game_account.state = GameState::Waiting;
    game_account.max_players = max_players;
    game_account.players = Vec::new();
    game_account.winners = winners;
    game_account.unix_timestamp_str = unix_timestamp;
    game_account.game_id = 0;
    game_account.mint = *ctx.accounts.mint.to_account_info().key;
    game_account.wager_type = wager_type;
    game_account.wager = wager;
    game_account.bump = *ctx.bumps.get("game_account").unwrap(); // anchor 0.28
    // game_account.bump = ctx.bumps.game_account; // anchor 0.29
    Ok(())
}