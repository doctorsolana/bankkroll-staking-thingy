use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer,},
};
use anchor_spl::token;

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game_account: Account<'info, Game>,

    #[account(mut, seeds = [b"GAMBA_STATE".as_ref()], bump)]
    pub gamba_state: Account<'info, GambaState>,

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

    //creator address
    /// CHECK: THIS IS FINE I THINK!!!
    pub creator_address: UncheckedAccount<'info>,

    //creator associated token account
    #[account(
        init_if_needed,
        payer = player_account,
        associated_token::mint = mint,
        associated_token::authority = creator_address)]
    pub creator_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn join_game_handler(
    ctx: Context<JoinGame>,
    creator_fee: u32,
    wager: u64,
) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;

    // If game is in SameWager state then set final_wager to the game_account's wager
    let mut final_wager = wager;
    if game_account.wager_type == WagerType::SameWager {
        final_wager = game_account.wager;
    }

    // calcualte creator fee amount from creator fee bip
    let creator_fee_amount = (final_wager * creator_fee as u64) / 10000;

    //gamba placeholder fee at 1%
    let gamba_fee_amount = (final_wager * ctx.accounts.gamba_state.gamba_fee_bps as u64) / 10000;

    let player = Player {
        creator_address_ata: *ctx.accounts.creator_ata.to_account_info().key,
        user_ata: *ctx.accounts.player_ata.to_account_info().key,
        creator_fee_amount: creator_fee_amount,
        gamba_fee_amount: gamba_fee_amount,
        wager: final_wager,
    };

    //check that the game is in waiting state
    if game_account.state != GameState::Waiting {
        return Err(GambaError::PlayerAlreadyInGame.into());
    };

    //check that the player is not already in the game
    for p in game_account.players.iter() {
        if p.user_ata == player.user_ata {
            return Err(GambaError::PlayerAlreadyInGame.into());
        }
    };

    //add the player to the game
    game_account.players.push(player);

    // transfer wager,createor_fee_amount and game fee to the game account token account
    let cpi_accounts = SplTransfer {
        from: ctx.accounts.player_ata.to_account_info(),
        to: ctx.accounts.game_account_ta.to_account_info(),
        authority: ctx.accounts.player_account.to_account_info(),
      };
      let cpi_program = ctx.accounts.token_program.to_account_info();
      let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
      token::transfer(cpi_ctx, final_wager + creator_fee_amount + gamba_fee_amount)?;

    //if max players is reached, start the game
    if game_account.players.len() == game_account.max_players as usize {
        game_account.state = GameState::Playing;
    }
    Ok(())
}