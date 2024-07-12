use anchor_lang::prelude::*;

declare_id!("acVeuAwGRVKa4i3aTurgynaYQGER9BieHbXTVwdRAWX");

#[program]
pub mod project2 {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
