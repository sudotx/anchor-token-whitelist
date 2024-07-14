use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};
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

    // Initialize the whitelist sale
    pub fn initialize_sale(
        ctx: Context<Initialize>,
        token_price: u64,
        purchase_limit: u64,
    ) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        whitelist_sale_account.admin = ctx.accounts.initializer.key();
        // set token price
        whitelist_sale_account.token_price = token_price;
        whitelist_sale_account.purchase_limit = purchase_limit;
        whitelist_sale_account.is_active = false;
        // only owner allowed to update prices
        // set a purchase limit per wallet
        // set up token

        // set purchase limit
        // initializing the whitelist
        Ok(())
    }

    // Allow a whitelisted user to purchase tokens, ensuring they do not exceed the purchase limit.
    pub fn purchase_tokens(ctx: Context<Purchase>, amount: u64) -> Result<()> {
        // check if user exists in the merkle root
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
        // This is a simplified version, you'll need to implement the actual transfer
        // using the System Program

        // Mint tokens to user
        // You'll need to implement token minting logic here

        // require!(whitelist_sale_account.is_active, ErrorCode::SaleNotActive);
        // send sol to vault
        // sol has to be up to the
        // mint tokens to user

        // ensure they dont surpass purchase limit

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

        // only admin allowed here
        // add a user to the whitelist
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
        // only admin allowed here
        // add a user to the whitelist
        Ok(())
    }
    // Allow anyone to check if an address is whitelisted.
    pub fn check_whitelist_status(
        ctx: Context<AddToWhitelist>,
        whitelist_address: Pubkey,
    ) -> Result<()> {
        // only admin allowed here
        // add a user to the whitelist
        Ok(())
    }

    // Allow the admin to update parameters of the sale, such as the token price or purchase limit.
    pub fn update_sale_parameters(
        ctx: Context<AddToWhitelist>,
        token_price: u64,
        purchase_limit: u64,
    ) -> Result<()> {
        // only admin allowed here

        // set token price and purchase limit
        Ok(())
    }
    // end sale period, preventing purchases
    pub fn end_sale(ctx: Context<AddToWhitelist>) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );
        // only admin allowed here
        // end the sale
        Ok(())
    }
    // start sale period
    pub fn start_sale(ctx: Context<AddToWhitelist>) -> Result<()> {
        let whitelist_sale_account = &mut ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );
        // only admin allowed here
        // end the sale
        Ok(())
    }
    // withdraw funds from the sale
    pub fn withdraw_funds(ctx: Context<AddToWhitelist>, amount: u64) -> Result<()> {
        let whitelist_sale_account = &ctx.accounts.whitelist_sale_account;
        require!(
            ctx.accounts.admin.key() == whitelist_sale_account.admin,
            ErrorCode::Unauthorized
        );

        // Implement the logic to transfer funds from the vault to the admin
        // This will involve using the System Program to transfer SOL

        // allow admin to wtihdraw funds
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
        4 + (32 * Self::MAX_WHITELIST_SIZE); // whitelist vec
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
    pub admin_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
