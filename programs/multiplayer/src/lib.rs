use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer},
};

declare_id!("EC6VgGSamTvY3XRuYC7uyW1DZMrvuRkfztdy6YeCfNxX");

#[program]
mod multiplayer {

    use anchor_spl::token; //idk why i neeed this import here when there is one above lets see
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, max_players: u32) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        game_account.game_maker = *ctx.accounts.game_maker.key;
        game_account.state = GameState::Waiting;
        game_account.max_players = max_players;
        game_account.players = Vec::new();
        game_account.game_id = 0;
        game_account.mint = *ctx.accounts.mint.to_account_info().key;
        // game_account.bump = [*ctx.bumps.get("game_account").unwrap()]; // anchor 0.28
        game_account.bump = ctx.bumps.game_account; // anchor 0.29
        Ok(())
    }

    pub fn join_game(
        ctx: Context<JoinLeaveGame>,
        creator_fee: u32,
        wager: u64,
    ) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;

        // calcualte creator fee amount from creator fee bip
        let creator_fee_amount = (wager * creator_fee as u64) / 10000;

        let player = Player {
            creator_address_ata: *ctx.accounts.creator_ata.to_account_info().key,
            user_ata: *ctx.accounts.player_ata.to_account_info().key,
            creator_fee_amount: creator_fee_amount,
            wager: wager,
        };

        //check that the game is in waiting state
        if game_account.state != GameState::Waiting {
            return Err(ErrorCode::PlayerAlreadyInGame.into());
        };

        //check that the player is not already in the game
        for p in game_account.players.iter() {
            if p.user_ata == player.user_ata {
                return Err(ErrorCode::PlayerAlreadyInGame.into());
            }
        }
        //add the player to the game
        game_account.players.push(player);

        // transfer wager and createor_fee_amount to the game account token account
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.player_ata.to_account_info(),
            to: ctx.accounts.game_account_ta.to_account_info(),
            authority: ctx.accounts.player_account.to_account_info(),
          };
          let cpi_program = ctx.accounts.token_program.to_account_info();
          let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
          token::transfer(cpi_ctx, wager + creator_fee_amount)?;

        //if max players is reached, start the game
        if game_account.players.len() == game_account.max_players as usize {
            game_account.state = GameState::Playing;
        }
        Ok(())
    }

    pub fn leave_game(ctx: Context<JoinLeaveGame>) -> Result<()> {
        // Ensure the game is in the correct state before proceeding.
        if ctx.accounts.game_account.state != GameState::Waiting {
            return Err(ErrorCode::GameInProgress.into());
        }
    
        // Attempt to find the player in the game.
        if let Some(index) = ctx.accounts.game_account.players.iter().position(|p| p.user_ata == *ctx.accounts.player_account.key) {
            // Calculate the total amount to transfer back (wager + creator fee).
            let total_amount = ctx.accounts.game_account.players[index].wager + ctx.accounts.game_account.players[index].creator_fee_amount;
    
            let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.max_players.to_le_bytes(), &[ctx.accounts.game_account.bump]];
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
            return Err(ErrorCode::PlayerNotInGame.into());
        }
    
        Ok(())
    }


    

    pub fn settle_game(ctx: Context<SettleGame>) -> Result<()> {
        let token_program = &ctx.accounts.token_program;

        let creator_atas = vec![
            &ctx.accounts.creator_1_ata,
            &ctx.accounts.creator_2_ata,
            // &ctx.accounts.creator_3_ata,
        ];
        
        for (i, player) in ctx.accounts.game_account.players.iter().enumerate() {
            if let Some(Some(creator_account)) = creator_atas.get(i) {
                let cpi_accounts = SplTransfer {
                    from: ctx.accounts.game_account_ta.to_account_info(),
                    to: creator_account.to_account_info(),
                    authority: ctx.accounts.game_account.to_account_info(),
                };
                let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.max_players.to_le_bytes(), &[ctx.accounts.game_account.bump]];
                let signer = &[&seeds[..]];
                let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer);
                token::transfer(cpi_ctx, player.creator_fee_amount)?;
            }
        }

        Ok(())
    }
    
}

#[derive(Accounts)]
#[instruction(max_players: u32)] // just testing
pub struct CreateGame<'info> {
    #[account(init, payer = game_maker, space = 8 + 1000, seeds = [b"GAME",game_maker.key().as_ref(), &max_players.to_le_bytes()], bump )] 
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

#[derive(Accounts)]
pub struct JoinLeaveGame<'info> {
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

    //creator address
    pub creator_address: UncheckedAccount<'info>,

    //creator associated token account
    #[account(mut, 
        associated_token::mint = mint,
        associated_token::authority = creator_address)]
    pub creator_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SettleGame<'info> {
    #[account(mut)]
    pub rng: Signer<'info>,

    #[account(mut, address = game_account.game_maker)]
    pub game_maker: AccountInfo<'info>,

    #[account(mut, close = game_maker)]
    pub game_account: Account<'info, Game>,

    #[account(mut, close = game_maker, seeds = [game_account.key().as_ref()], bump)]
    pub game_account_ta: Account<'info, TokenAccount>,

    #[account(address = game_account.mint)]
    pub mint: Account<'info, Mint>,

    // Plyaer 1
    #[account(mut, address = game_account.players[0].user_ata)]
    pub player_1_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, address = game_account.players[0].creator_address_ata)]
    pub creator_1_ata: Option<Account<'info, TokenAccount>>,

    // Plyaer 2
    #[account(mut, address = game_account.players[1].user_ata)]
    pub player_2_ata: Option<Account<'info, TokenAccount>>,

    #[account(mut, address = game_account.players[1].creator_address_ata)]
    pub creator_2_ata: Option<Account<'info, TokenAccount>>,

    // // Plyaer 3
    // #[account(mut, address = game_account.players[2].user_ata)]
    // pub player_3_ata: Option<Account<'info, TokenAccount>>,

    // #[account(mut, address = game_account.players[2].creator_address_ata)]
    // pub creator_3_ata: Option<Account<'info, TokenAccount>>,

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
    pub bump: u8,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Player {
    creator_address_ata: Pubkey,
    user_ata: Pubkey,
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
    #[msg("Player is not in the game")]
    PlayerNotInGame,
    #[msg("Game is already in progress")]
    GameInProgress,
    #[msg("Invalid game account")]
    InvalidGameAccount,
}