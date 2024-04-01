use crate::*;
use anchor_spl::token::{Mint, Token, TokenAccount,};

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // just the creator of the vault

    pub mint: Account<'info, Mint>, //mint account

    #[account(init, payer = signer, space = 200, seeds = [b"VAULT", mint.key().as_ref()], bump )] // space is placeholder, perhaps want different seed for making many vaults
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = vault, 
        seeds = [vault.key().as_ref()], bump
          )]
    pub vault_ta: Account<'info, TokenAccount>, // normal token account for vault

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create_game_handler(ctx: Context<CreateVault>) -> Result<()> {
    // let clock = Clock::get()?; 
    // let current_timestamp = clock.unix_timestamp;
    // let game_account = &mut ctx.accounts.game_account;

    // // if u8 is more than 0 then it is a custom wager
    // let wager_type = if wager_type > 0 { WagerType::CustomWager } else { WagerType::SameWager };

    // game_account.game_maker = *ctx.accounts.game_maker.key;
    // game_account.state = GameState::Waiting;
    // game_account.mint = *ctx.accounts.mint.to_account_info().key;
    // game_account.max_players = max_players;
    // game_account.players = Vec::new();
    // game_account.winners = winners;
    // game_account.game_id = 0;
    // game_account.game_expiration_timestamp = current_timestamp + duration_seconds;
    // game_account.unique_identifier = unique_identifier;
    // game_account.wager_type = wager_type;
    // game_account.wager = wager;
    // game_account.bump = *ctx.bumps.get("game_account").unwrap(); // anchor 0.28
    // game_account.bump = ctx.bumps.game_account; // anchor 0.29

    Ok(())
}


