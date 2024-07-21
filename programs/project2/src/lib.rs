use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

declare_id!("acVeuAwGRVKa4i3aTurgynaYQGER9BieHbXTVwdRAWX");

const MAX_WHITELIST_SIZE: usize = 1000;

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Sale is not active")]
    SaleNotActive,
    #[msg("Address not whitelisted")]
    NotWhitelisted,
    #[msg("Numerical overflow")]
    NumericalOverflow,
}

#[program]
pub mod whitelist_sale {
    use super::*;
    use crate::instruction::StartSale;

    // Initialize the whitelist sale
    pub fn initialize_sale(
        ctx: Context<Initialize>,
        token_price: u64,
        purchase_limit: u64,
    ) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        whitelist_sale_account.admin = ctx.accounts.initializer.key();
        whitelist_sale_account.token_price = token_price;
        whitelist_sale_account.purchase_limit = purchase_limit;
        whitelist_sale_account.is_active = false;
        whitelist_sale_account.total_tokens_sold = 0;
        whitelist_sale_account.whitelist = vec![];

        Ok(())
    }

    // Allow a whitelisted user to purchase tokens, ensuring they do not exceed the purchase limit.
    pub fn purchase_tokens(ctx: Context<Purchase>, amount: u64) -> Result<()> {
        let whitelist_sale_account = &ctx.accounts.whitelist_sale_account;

        require!(whitelist_sale_account.is_active, ErrorCode::SaleNotActive);
        require!(
            whitelist_sale_account
                .whitelist
                .contains(&ctx.accounts.buyer.key()),
            ErrorCode::NotWhitelisted
        );

        let total_cost = amount
            .checked_mul(whitelist_sale_account.token_price)
            .ok_or(ErrorCode::NumericalOverflow)?;

        // Transfer SOL from buyer to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.buyer.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), total_cost)?;

        // Mint tokens to user
        let cpi_accounts_mint = MintTo {
            mint: ctx.accounts.mint.to_account_info().clone(),
            to: ctx.accounts.buyer_token_account.to_account_info().clone(),
            authority: ctx.accounts.mint_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::mint_to(CpiContext::new(cpi_program, cpi_accounts_mint), amount)?;

        // Ensure they do not surpass the purchase limit
        let new_total = whitelist_sale_account
            .total_tokens_sold
            .checked_add(amount)
            .ok_or(ErrorCode::NumericalOverflow)?;

        require!(
            new_total <= whitelist_sale_account.purchase_limit,
            ErrorCode::NumericalOverflow
        );

        whitelist_sale_account.total_tokens_sold = new_total;

        Ok(())
    }

    // Allow the admin to add an address to the whitelist.
    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>, whitelist_address: Pubkey) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;

        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        if !whitelist_sale_account
            .whitelist
            .contains(&whitelist_address)
        {
            whitelist_sale_account.whitelist.push(whitelist_address);
        }

        Ok(())
    }

    // Allow the admin to remove an address from the whitelist.
    pub fn remove_from_whitelist(
        ctx: Context<RemoveFromWhitelist>,
        whitelist_address: Pubkey,
    ) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        whitelist_sale_account
            .whitelist
            .retain(|&address| address != whitelist_address);

        Ok(())
    }
    // Allow anyone to check if an address is whitelisted.
    pub fn check_whitelist_status(
        ctx: Context<CheckWhitelistStatus>,
        whitelist_address: Pubkey,
    ) -> Result<bool> {
        let whitelist_sale_account = &ctx.accounts.whitelist_sale_account;
        Ok(whitelist_sale_account
            .whitelist
            .contains(&whitelist_address))
    }

    // Allow the admin to update parameters of the sale, such as the token price or purchase limit.
    pub fn update_sale_parameters(
        ctx: Context<UpdateSaleParameters>,
        token_price: u64,
        purchase_limit: u64,
    ) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        whitelist_sale_account.token_price = token_price;
        whitelist_sale_account.purchase_limit = purchase_limit;

        Ok(())
    }
    // end sale period, preventing purchases
    pub fn end_sale(ctx: Context<EndSale>) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        whitelist_sale_account.is_active = false;

        Ok(())
    }

    // start sale period
    pub fn start_sale(ctx: Context<StartSale>) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        whitelist_sale_account.is_active = true;

        Ok(())
    }
    // withdraw funds from the sale
    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
        let whitelist_sale_account = &ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        // Implement the logic to transfer funds from the vault to the admin
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.admin.to_account_info().clone(),
            authority: ctx
                .accounts
                .whitelist_sale_account
                .to_account_info()
                .clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(init, payer = initializer, space = 8 + WhitelistSaleAccount::LEN)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub mint_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
}

#[account]
pub struct WhitelistSaleAccount {
    pub admin: Pubkey,
    pub token_price: u64,
    pub purchase_limit: u64,
    pub is_active: bool,
    pub total_tokens_sold: u64,
    pub whitelist: Vec<Pubkey>,
}

impl WhitelistSaleAccount {
    pub const MAX_WHITELIST_SIZE: usize = 1000;
    pub const LEN: usize = 8 + // discriminator
        32 + // admin pubkey
        8 + // token_price
        8 + // purchase_limit
        1 + // is_active
        8 + // total_tokens_sold
        4 + (32 * Self::MAX_WHITELIST_SIZE);
}

#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
}

#[derive(Accounts)]
pub struct CheckWhitelistStatus<'info> {
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
}

#[derive(Accounts)]
pub struct UpdateSaleParameters<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
}

#[derive(Accounts)]
pub struct EndSale<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub whitelist_sale_account: Account<'info, WhitelistSaleAccount>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
