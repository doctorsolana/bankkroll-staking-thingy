use crate::*;
use anchor_spl::token::{Mint, Token, TokenAccount,};

#[derive(Accounts)]
#[instruction(max_players: u8, winners: u8, duration_seconds: i64, unique_identifier: u32, wager_type: WagerType, wager: u64)] 
pub struct CreateGame<'info> {
    #[account(init, payer = game_maker, space = 8 + 1000, seeds = [b"GAME",game_maker.key().as_ref(), &unique_identifier.to_le_bytes()], bump )] // use unix timestamp as seed
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

pub fn create_game_handler(ctx: Context<CreateGame>, max_players: u8, winners: u8, duration_seconds: i64, unique_identifier: u32, wager_type: u8, wager: u64) -> Result<()> {
    let clock = Clock::get()?; 
    let current_timestamp = clock.unix_timestamp;
    let game_account = &mut ctx.accounts.game_account;

    // if u8 is more than 0 then it is a custom wager
    let wager_type = if wager_type > 0 { WagerType::CustomWager } else { WagerType::SameWager };

    game_account.game_maker = *ctx.accounts.game_maker.key;
    game_account.state = GameState::Waiting;
    game_account.mint = *ctx.accounts.mint.to_account_info().key;
    game_account.max_players = max_players;
    game_account.players = Vec::new();
    game_account.winners = winners;
    game_account.game_id = 0;
    game_account.game_expiration_timestamp = current_timestamp + duration_seconds;
    game_account.unique_identifier = unique_identifier;
    game_account.wager_type = wager_type;
    game_account.wager = wager;
    game_account.bump = *ctx.bumps.get("game_account").unwrap(); // anchor 0.28
    // game_account.bump = ctx.bumps.game_account; // anchor 0.29

    msg!("wager_amount: {}", wager);
    Ok(())
}


