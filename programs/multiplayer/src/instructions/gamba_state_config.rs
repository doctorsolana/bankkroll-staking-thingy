use crate::*;

#[derive(Accounts)]
pub struct GambaConfig<'info> {
    #[account(init_if_needed, payer = authority, space = 100, seeds = [b"GAMBA_STATE".as_ref()], bump)]
    pub gamba_state: Account<'info, GambaState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn gamba_config_handler(ctx: Context<GambaConfig>, gamba_fee: u32, rng: Pubkey) -> Result<()> {
    let gamba_state = &mut ctx.accounts.gamba_state;

    // if already initialized check that authority is the same otherwise throw error
    if gamba_state.initialized {
        if gamba_state.authority != *ctx.accounts.authority.key {
            return Err(GambaError::AuthorityMismatch.into());
        }
    }

    gamba_state.rng = rng;
    gamba_state.authority = *ctx.accounts.authority.key;
    gamba_state.gamba_fee_bps = gamba_fee;

    gamba_state.initialized = true;
    gamba_state.bump = *ctx.bumps.get("gamba_state").unwrap(); // anchor 0.28
    // gamba_state.bump = ctx.bumps.gamba_state; // anchor 0.29

    msg!("Gamba state! Gamba_fee: {}, rng_address {}", gamba_fee, rng.to_string());
    Ok(())
}


