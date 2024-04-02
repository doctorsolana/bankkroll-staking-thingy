use crate::*;
use anchor_spl::token::{Mint, Token, TokenAccount,};

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // just the creator of the vault

    pub mint: Account<'info, Mint>, //mint account

    #[account(init, payer = signer, space = 200, seeds = [b"VAULT", mint.key().as_ref()], bump )] // space is placeholder, perhaps you want unique seed for making many vaults with same mint
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

pub fn create_vault_handler(ctx: Context<CreateVault>) -> Result<()> {

    let vault = &mut ctx.accounts.vault;

    vault.mint = *ctx.accounts.mint.to_account_info().key;
    vault.token_account = *ctx.accounts.vault_ta.to_account_info().key;
    vault.bump = *ctx.bumps.get("vault").unwrap(); // anchor 0.28
    // game_account.bump = ctx.bumps.game_account; // anchor 0.29

    Ok(())
}


