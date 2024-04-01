use crate::*;
use anchor_spl::token::{Mint, Token, TokenAccount,};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // just the creator of the vault

    #[account(init_if_needed, payer = signer, space = 200, seeds = [b"VAULT"], bump)]
    pub user_account: Account<'info, UserAccount>,

    #[account(address = vault.mint)] // make sure mint is the same as saved in vault
    pub mint: Account<'info, Mint>, 

    #[account(seeds = [b"VAULT", mint.key().as_ref()], bump )] // seed aint super importnat here
    pub vault: Account<'info, Vault>,

    #[account(mut, seeds = [vault.key().as_ref()], bump)] // seeds make sure its the right token account
    pub vault_ta: Account<'info, TokenAccount>, 

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn deposit_handler(ctx: Context<Deposit>) -> Result<()> {

    let vault = &mut ctx.accounts.vault;

    vault.mint = *ctx.accounts.mint.key();
    vault.token_account = *ctx.accounts.vault_ta.key();
    vault.bump = *ctx.bumps.get("vault").unwrap(); // anchor 0.28
    // game_account.bump = ctx.bumps.game_account; // anchor 0.29

    Ok(())
}


