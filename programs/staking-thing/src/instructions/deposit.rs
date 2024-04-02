use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer,},
};
use anchor_spl::token;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // the user who is depositing

    #[account(init_if_needed, payer = signer, space = 200, seeds = [signer.key().as_ref(), vault.key().as_ref()], bump)]
    // space is placeholder, seeds make sure only one user account per user per vault
    pub user_account: Account<'info, UserAccount>,

    #[account(address = vault.mint)] // make sure mint is the same as saved in vault
    pub mint: Account<'info, Mint>,

    #[account(mut, seeds = [b"VAULT", mint.key().as_ref()], bump )]
    // seed aint super importnat here but can be nice to force it
    pub vault: Account<'info, Vault>,

    //player associated token account
    #[account(mut, 
        associated_token::mint = mint,
        associated_token::authority = signer,)]
    pub signer_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [vault.key().as_ref()], bump)]
    // seeds make sure its the right token account but also not essential
    pub vault_ta: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn deposit_handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;

    //if its the first time set all the fields
    if !user_account.initialized {
        user_account.owner = *ctx.accounts.signer.to_account_info().key;
        user_account.vault = *ctx.accounts.vault.to_account_info().key;
        user_account.lp = 0;
        user_account.staking_end = 0; // idea would be to play around with this depening on what you want
        user_account.initialized = true; // very important to set this to true so we dont overwrite the account
        user_account.bump = *ctx.bumps.get("user_account").unwrap(); // anchor 0.28
        // user_account.bump = ctx.bumps.user_account; // anchor 0.29
    }

    // Calculate LP tokens to mint for the deposit based on the current ratio without floating-point arithmetic
    let total_vault_tokens = ctx.accounts.vault_ta.amount; // Total tokens in the vault
    let total_lp_issued = ctx.accounts.vault.total_lp; // Total LP tokens issued

    // Ensure we are not dividing by zero
    let lp_tokens_for_deposit: u64 = if total_lp_issued > 0 {
        // Calculate the amount of LP tokens to issue for the deposited amount in a safe way THIS IS SAFE AFAIK (no overflow)
        (amount as u128)
            .checked_mul(total_lp_issued as u128)
            .unwrap()
            .checked_div(total_vault_tokens as u128)
            .unwrap()
            .try_into()
            .unwrap()
    } else {
        // If no LP tokens have been issued yet, initialize ratio (e.g., 1:1)
        amount 
    };

    ctx.accounts.vault.total_lp += lp_tokens_for_deposit; // since we dont issue a lp token we just keep track of total supply in the vault
    user_account.lp += lp_tokens_for_deposit; // instead of sending out LP tokens we just keep track of them in the user account

    // transfer tokens from user to vault
    let cpi_accounts = SplTransfer {
        from: ctx.accounts.signer_ata.to_account_info(),
        to: ctx.accounts.vault_ta.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}
