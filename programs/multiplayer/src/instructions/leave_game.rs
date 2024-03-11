use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer,},
};
use anchor_spl::token;


#[derive(Accounts)]
pub struct LeaveGame<'info> {
    #[account(mut)]
    pub game_account: Account<'info, Game>,

    //game associated token account
    #[account(mut, seeds = [game_account.key().as_ref()], bump)]
    pub game_account_ta: Account<'info, TokenAccount>,

    //mint account
    #[account(address = game_account.mint)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub player_account: Signer<'info>,

    //player associated token account
    #[account(mut, 
        associated_token::mint = mint,
        associated_token::authority = player_account,)]
    pub player_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn leave_game_handler(ctx: Context<LeaveGame>) -> Result<()> {
    // Ensure the game is in the correct state before proceeding.
    if ctx.accounts.game_account.state != GameState::Waiting {
        return Err(GambaError::GameInProgress.into());
    }

    // Attempt to find the player in the game.
    if let Some(index) = ctx.accounts.game_account.players.iter().position(|p| p.user_ata == ctx.accounts.player_ata.key()) {
        // Calculate the total amount to transfer back (wager + creator fee).
        let total_amount = ctx.accounts.game_account.players[index].wager + ctx.accounts.game_account.players[index].creator_fee_amount + ctx.accounts.game_account.players[index].gamba_fee_amount;	

        let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.unix_timestamp_str.as_ref(), &[ctx.accounts.game_account.bump]];
        let signer = &[&seeds[..]];

        // Set up the transfer CPI with the PDA as the authority.
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.game_account_ta.to_account_info(),
            to: ctx.accounts.player_ata.to_account_info(),
            authority: ctx.accounts.game_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, total_amount)?;

        // Remove the player from the game.
        ctx.accounts.game_account.players.remove(index);
    } else {
        // Player not found in the game.
        return Err(GambaError::PlayerNotInGame.into());
    }

    Ok(())
}