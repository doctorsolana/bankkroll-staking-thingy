use crate::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // the user who is depositing

    #[account(init_if_needed, payer = signer, space = 200, seeds = [signer.key().as_ref(), vault.key().as_ref()], bump)]
    // space is placeholder, seeds make sure only one user account per user per vault
    pub user_account: Account<'info, UserAccount>,

    #[account(address = vault.mint)] // make sure mint is the same as saved in vault
    pub mint: Account<'info, Mint>,

    #[account(mut, seeds = [b"VAULT", mint.key().as_ref()], bump )] // seed aint super importnat here
    pub vault: Account<'info, Vault>,

    #[account(mut, seeds = [vault.key().as_ref()], bump)]
    // seeds make sure its the right token account
    pub vault_ta: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn deposit_handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;

    user_account.owner = *ctx.accounts.signer.key;
    user_account.vault = ctx.accounts.vault.key();
    user_account.lp = 0;
    user_account.staking_end = 0;
    user_account.initialized = true;
    user_account.bump = *ctx.bumps.get("user_account").unwrap(); // anchor 0.28
    // user_account.bump = ctx.bumps.user_account; // anchor 0.29

    Ok(())
}
