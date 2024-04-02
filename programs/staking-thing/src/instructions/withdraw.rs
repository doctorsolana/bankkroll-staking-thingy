use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer,},
};
use anchor_spl::token;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // the user who is depositing

    #[account(mut, seeds = [signer.key().as_ref(), vault.key().as_ref()], bump)]
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

pub fn withdraw_handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let vault = &mut ctx.accounts.vault;

    // Assume total_vault_tokens represents the amount of underlying tokens the vault holds.
    let total_vault_tokens = ctx.accounts.vault_ta.amount; 
    let total_lp_issued = vault.total_lp; 

    // Ensure we are not dividing by zero and calculate the amount of underlying tokens to withdraw
    if total_lp_issued == 0 {
        return Err(ProgramError::DivideByZero.into());
    }
    
    let amount_to_withdraw: u64 = (lp_tokens_to_redeem as u128)
        .checked_mul(total_vault_tokens as u128)
        .ok_or(ProgramError::CalculationFailure)?
        .checked_div(total_lp_issued as u128)
        .ok_or(ProgramError::CalculationFailure)?
        .try_into()
        .map_err(|_| ProgramError::CalculationFailure)?;
    
    // Update LP and total_lp in vault
    user_account.lp = user_account.lp.checked_sub(lp_tokens_to_redeem)
        .ok_or(ProgramError::InsufficientFunds)?;
    vault.total_lp = vault.total_lp.checked_sub(lp_tokens_to_redeem)
        .ok_or(ProgramError::InsufficientFunds)?;
    
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
