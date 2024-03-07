use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer, CloseAccount},
};
use anchor_spl::token;

declare_id!("DgVtNY9ZPv68mmj69Hxo4pFTwo4RxfdCziy8irHj8dzp");

#[program]
mod multiplayer {

    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, max_players: u32) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        game_account.game_maker = *ctx.accounts.game_maker.key;
        game_account.state = GameState::Waiting;
        game_account.max_players = max_players;
        game_account.players = Vec::new();
        game_account.created_at = Clock::get()?.unix_timestamp;
        game_account.game_id = 0;
        game_account.mint = *ctx.accounts.mint.to_account_info().key;
        game_account.bump = *ctx.bumps.get("game_account").unwrap(); // anchor 0.28
        // game_account.bump = ctx.bumps.game_account; // anchor 0.29
        Ok(())
    }

    pub fn join_game(
        ctx: Context<JoinGame>,
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

    pub fn leave_game(ctx: Context<LeaveGame>) -> Result<()> {
        // Ensure the game is in the correct state before proceeding.
        if ctx.accounts.game_account.state != GameState::Waiting {
            return Err(ErrorCode::GameInProgress.into());
        }
    
        // Attempt to find the player in the game.
        if let Some(index) = ctx.accounts.game_account.players.iter().position(|p| p.user_ata == ctx.accounts.player_ata.key()) {
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

        //log how many tokens the game token account has
        let amount = &ctx.accounts.game_account_ta.amount;
        
        msg!("Game Account Token Account has {} tokens", amount);

        let player_atas = vec![
            &ctx.accounts.player_1_ata,
            &ctx.accounts.player_2_ata,
            // &ctx.accounts.player_3_ata,
        ];

        let creator_atas = vec![
            &ctx.accounts.creator_1_ata,
            &ctx.accounts.creator_2_ata,
            // &ctx.accounts.creator_3_ata,
        ];

        // Initialize the total wager amount
        let mut total_wager: u64 = 0;
        
        for (i, player) in ctx.accounts.game_account.players.iter().enumerate() {
            if let Some(Some(creator_account)) = creator_atas.get(i) {
                // Transfer creator fee to each creator's account
                let cpi_accounts = SplTransfer {
                    from: ctx.accounts.game_account_ta.to_account_info(),
                    to: creator_account.to_account_info(),
                    authority: ctx.accounts.game_account.to_account_info(),
                };
                let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.max_players.to_le_bytes(), &[ctx.accounts.game_account.bump]];
                let signer = &[&seeds[..]];
                let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer);
                token::transfer(cpi_ctx, player.creator_fee_amount)?;

                // log the creator fee amount
                msg!("Creator Fee Amount: {}", player.creator_fee_amount);
            }
    
            if let Some(Some(player_account)) = player_atas.get(i) {
                // Sum up total wager amount from each player's account
                total_wager += player.wager;
            }
        }

        //log total wager amount
        msg!("Total Wager Amount: {}", total_wager);

        // Transfer total wager amount to the winner's token account
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.game_account_ta.to_account_info(),
            to: ctx.accounts.winner_ata.to_account_info(),
            authority: ctx.accounts.game_account.to_account_info(),
        };
        let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.max_players.to_le_bytes(), &[ctx.accounts.game_account.bump]];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer);
        token::transfer(cpi_ctx, total_wager)?;


        // After settling the game, close the game_account_ta and send the rent to the game_maker
        let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.max_players.to_le_bytes(), &[ctx.accounts.game_account.bump]];
        let signer_seeds = &[&seeds[..]];

        // Close the token account and send the remaining SOL to the game_maker
        let cpi_accounts_close_ta = CloseAccount {
            account: ctx.accounts.game_account_ta.to_account_info(),
            destination: ctx.accounts.game_maker.to_account_info(),
            authority: ctx.accounts.game_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let close_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts_close_ta, signer_seeds);
        token::close_account(close_ctx)?;

        // Transfering all the SOL out of the game_account to the game_maker effectively closes it? but i dont think you can ever open it again???
        **ctx.accounts.game_maker.to_account_info().try_borrow_mut_lamports()? += ctx.accounts.game_account.to_account_info().lamports();
        **ctx.accounts.game_account.to_account_info().try_borrow_mut_lamports()? = 0;


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
pub struct JoinGame<'info> {
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
    /// CHECK: THIS IS FINE
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

#[derive(Accounts)]
pub struct SettleGame<'info> {
    #[account(mut)]
    pub rng: Signer<'info>,

    #[account(mut, address = game_account.game_maker)]
    /// CHECK: THIS IS FINE
    pub game_maker: AccountInfo<'info>,

    #[account(mut)]
    pub game_account: Account<'info, Game>,

    #[account(mut, seeds = [game_account.key().as_ref()], bump)]
    pub game_account_ta: Account<'info, TokenAccount>,

    #[account(address = game_account.mint)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub winner_ata: Account<'info, TokenAccount>,

    // Plyaer 1
    #[account(mut, address = game_account.players[0].user_ata)]
    pub player_1_ata: Option<Box<Account<'info, TokenAccount>>>,

    #[account(mut, address = game_account.players[0].creator_address_ata)]
    pub creator_1_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Plyaer 2
    #[account(mut, address = game_account.players[1].user_ata)]
    pub player_2_ata: Option<Box<Account<'info, TokenAccount>>>,

    #[account(mut, address = game_account.players[1].creator_address_ata)]
    pub creator_2_ata: Option<Box<Account<'info, TokenAccount>>>,

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
    pub created_at: i64,
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