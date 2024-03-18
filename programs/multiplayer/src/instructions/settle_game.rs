use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{CloseAccount, Mint, Token, TokenAccount, Transfer as SplTransfer}
};
use anchor_spl::token;

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

    // Gamba Fee Account
    #[account(mut)] // make sure this is initialized
    pub gamba_fee_ata: Account<'info, TokenAccount>,// add constraints later based on gamba state account or whatever

    // Player 1
    #[account(mut, address = game_account.players[0].user_ata)]
    pub player_1_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[0].creator_address_ata)]
    pub creator_1_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 2
    #[account(mut, address = game_account.players[1].user_ata)]
    pub player_2_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[1].creator_address_ata)]
    pub creator_2_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 3
    #[account(mut, address = game_account.players[2].user_ata)]
    pub player_3_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[2].creator_address_ata)]
    pub creator_3_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 4
    #[account(mut, address = game_account.players[3].user_ata)]
    pub player_4_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[3].creator_address_ata)]
    pub creator_4_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 5
    #[account(mut, address = game_account.players[4].user_ata)]
    pub player_5_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[4].creator_address_ata)]
    pub creator_5_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 6
    #[account(mut, address = game_account.players[5].user_ata)]
    pub player_6_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[5].creator_address_ata)]
    pub creator_6_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 7
    #[account(mut, address = game_account.players[6].user_ata)]
    pub player_7_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[6].creator_address_ata)]
    pub creator_7_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 8
    #[account(mut, address = game_account.players[7].user_ata)]
    pub player_8_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[7].creator_address_ata)]
    pub creator_8_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 9
    #[account(mut, address = game_account.players[8].user_ata)]
    pub player_9_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[8].creator_address_ata)]
    pub creator_9_ata: Option<Box<Account<'info, TokenAccount>>>,

    // Player 10
    #[account(mut, address = game_account.players[9].user_ata)]
    pub player_10_ata: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, address = game_account.players[9].creator_address_ata)]
    pub creator_10_ata: Option<Box<Account<'info, TokenAccount>>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}


pub fn calculate_payouts(total_pot: u64, num_winners: usize) -> Vec<u64> {
    let mut payouts = vec![0; num_winners];

    //FLOATS MIGHT BE FINE HERE CUS ITS NOT SUPER IMPORTANT THAT ITS EXACT
    
    // Common ratio, r = 2 for doubling
    let r: f64 = 2.0;
    
    // Calculate the sum of geometric series to find 'a'
    // S = a * (1 - r^n) / (1 - r), solving for 'a' gives us:
    let a = total_pot as f64 / ((1.0 - r.powi(num_winners as i32)) / (1.0 - r));
    
    // Calculate each winner's payout
    for i in 0..num_winners {
        // The payout is 'a' times 'r' raised to the power of the winner's position (reversed since top winner gets more)
        payouts[i] = (a * r.powi((num_winners - i - 1) as i32)) as u64;
    }
    
    // Adjust for rounding errors by ensuring the total payouts equal the total pot
    let sum_payouts: u64 = payouts.iter().sum();
    if sum_payouts != total_pot {
        let diff = total_pot as i64 - sum_payouts as i64;
        // Adjust the last winner's payout to make up for rounding difference
        payouts[num_winners - 1] = (payouts[num_winners - 1] as i64 + diff) as u64;
    }
    
    payouts
}

// fake random shuffle made by gpt
pub fn pseudo_shuffle(vec: &mut [usize], seed: u64) {
    let mut rng_seed = seed;
    for i in (1..vec.len()).rev() {
        rng_seed = rng_seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB);
        let j = (rng_seed as usize) % (i + 1);
        vec.swap(i, j);
    }
}

pub fn settle_game_handler(ctx: Context<SettleGame>) -> Result<()> {

    let clock = Clock::get()?; // Get the current on-chain time
    let current_timestamp = clock.unix_timestamp;

    // The game can be settled if it's in the Playing state OR the current time is beyond the expiration timestamp
    if !(ctx.accounts.game_account.state == GameState::Playing || current_timestamp > ctx.accounts.game_account.game_expiration_timestamp) {
        msg!("Game cannot be settled at this time.");
        return Err(GambaError::CannotSettleYet.into());
    }

    let token_program = &ctx.accounts.token_program;

    //log how many tokens the game token account has
    let amount = &ctx.accounts.game_account_ta.amount;
    
    msg!("Game Account Token Account has {} tokens", amount);

    // Filter player_atas and creator_atas to only include Some values
    let filtered_player_atas: Vec<&Box<Account<TokenAccount>>> = vec![
        &ctx.accounts.player_1_ata,
        &ctx.accounts.player_2_ata,
        &ctx.accounts.player_3_ata,
        &ctx.accounts.player_4_ata,
        &ctx.accounts.player_5_ata,
        &ctx.accounts.player_6_ata,
        &ctx.accounts.player_7_ata,
        &ctx.accounts.player_8_ata,
        &ctx.accounts.player_9_ata,
        &ctx.accounts.player_10_ata,
    ].into_iter().filter_map(|ata| ata.as_ref()).collect();

    let filtered_creator_atas: Vec<&Box<Account<TokenAccount>>> = vec![
        &ctx.accounts.creator_1_ata,
        &ctx.accounts.creator_2_ata,
        &ctx.accounts.creator_3_ata,
        &ctx.accounts.creator_4_ata,
        &ctx.accounts.creator_5_ata,
        &ctx.accounts.creator_6_ata,
        &ctx.accounts.creator_7_ata,
        &ctx.accounts.creator_8_ata,
        &ctx.accounts.creator_9_ata,
        &ctx.accounts.creator_10_ata,
    ].into_iter().filter_map(|ata| ata.as_ref()).collect();

    msg!("Filtered Player ATAs: {:?}", filtered_player_atas.len());

    // Initialize the total wager amount
    let mut total_wager: u64 = 0;
    let mut total_gamba_fee: u64 = 0;

    //transfer creator fees to each creator
    for (i, player) in ctx.accounts.game_account.players.iter().enumerate() {
        if let Some(creator_account) = filtered_creator_atas.get(i) {
            let cpi_accounts = SplTransfer {
                from: ctx.accounts.game_account_ta.to_account_info(),
                to: creator_account.to_account_info(),
                authority: ctx.accounts.game_account.to_account_info(),
            };
            let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.unique_identifier.to_le_bytes(), &[ctx.accounts.game_account.bump]];
            let signer = &[&seeds[..]];
            let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer);
            token::transfer(cpi_ctx, player.creator_fee_amount)?;
        }

        // Calculate total wager using all players
        total_wager += player.wager;
        total_gamba_fee += player.gamba_fee_amount;
    }

    //log total wager amount
    msg!("Total Wager Amount: {}", total_wager);

    //check that winners are not more than players otherwise set number of winners to players
    let num_players = filtered_player_atas.len();
    let intended_winners = ctx.accounts.game_account.winners as usize;
    let effective_num_winners = if intended_winners > num_players { num_players } else { intended_winners };

    // Shuffle player indices based on the timestamp for pseudorandomness
    let timestamp = Clock::get()?.unix_timestamp as u64;
    let mut player_indices: Vec<usize> = (0..num_players).collect();
    pseudo_shuffle(&mut player_indices, timestamp); // fake randomness

    // Select the top N indices as winners, where N is the effective number of winners
    let winner_indices: Vec<usize> = player_indices.into_iter().take(effective_num_winners).collect();

    // Calculate payouts
    let payouts = calculate_payouts(total_wager, effective_num_winners);

    msg!("Payouts: {:?}", payouts);
    msg!("Winner Indices: {:?}", winner_indices);
    msg!("Effective Number of Winners: {}", effective_num_winners);

    // Distribute the calculated payouts to the winners
    for (i, &winner_index) in winner_indices.iter().enumerate() {
        let payout_amount = payouts[i];
        let winner_ata = filtered_player_atas[winner_index]; // This gets the Account<TokenAccount> for the winner

        msg!("Winner ATA: {:?}", winner_ata.to_account_info().key());
        msg!("Payout Amount: {}", payout_amount);
        
        // Perform the token transfer for the payout amount to the winner's token account
        let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.unique_identifier.to_le_bytes(), &[ctx.accounts.game_account.bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.game_account_ta.to_account_info(),
            to: winner_ata.to_account_info(),
            authority: ctx.accounts.game_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer);
        token::transfer(cpi_ctx, payout_amount)?;
    }

    // Transfer total gamba fee to the gamba fee account
    let cpi_accounts = SplTransfer {
        from: ctx.accounts.game_account_ta.to_account_info(),
        to: ctx.accounts.gamba_fee_ata.to_account_info(),
        authority: ctx.accounts.game_account.to_account_info(),
    };
    let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.unique_identifier.to_le_bytes(), &[ctx.accounts.game_account.bump]];
    let signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer);
    token::transfer(cpi_ctx, total_gamba_fee)?;

    // After settling the game, close the game_account_ta and send the rent to the game_maker
    let seeds = &[&b"GAME"[..], ctx.accounts.game_account.game_maker.as_ref(), &ctx.accounts.game_account.unique_identifier.to_le_bytes(), &[ctx.accounts.game_account.bump]];
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