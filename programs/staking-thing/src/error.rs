use anchor_lang::prelude::*;

#[error_code]
pub enum GambaError {
    #[msg("Cant unstake before staking period ends")]
    CantUnstakeBeforeStakingPeriodEnds,
}