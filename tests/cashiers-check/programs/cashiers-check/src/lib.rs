//! A cashiers check example. The funds are immediately withdrawn from a user's
//! account and sent to a program controlled `Check` account, where the funds
//! reside until they are "cashed" by the intended recipient. The creator of
//! the check can cancel the check at any time to get back the funds.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod cashiers_check {
    use super::*;

    #[access_control(CreateCheck::accounts(&ctx, nonce))]
    pub fn create_check(
        ctx: Context<CreateCheck>,
        amount: u64,
        memo: Option<String>,
        nonce: u8,
    ) -> Result<()> {
        // Transfer funds to the check.
        // so this is a struct that represents the set of accounts required for a transfer
        // these accounts are then fed into a CPI context which is used to actually invoke the transfer
        //so we are moving tokens from the "from" account, to the vault, and apparently the owner has authority over the from
        //enforced by the has_one constraint
        //this is a basic transfer for a token account, not a base sol account
        //so it's doing the transfer with the spl tokens inside the account, not lamports. i believe
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Print the check.
        let check = &mut ctx.accounts.check;
        check.amount = amount;
        check.from = *ctx.accounts.from.to_account_info().key;
        check.to = *ctx.accounts.to.to_account_info().key;
        check.vault = *ctx.accounts.vault.to_account_info().key;
        check.nonce = nonce;
        check.memo = memo;

        Ok(())
    }

    /*
    https://docs.rs/anchor-spl/0.16.1/src/anchor_spl/token.rs.html#13-35


    this is where you can see that it's calling the spl_token::transfer instruction
    
    so i'm guessing the reason why there's no anchor implementation for the transfer is because it's simple enough
    that u wouldn't want to use it??


    now with this 
    https://github.com/solana-labs/solana-program-library/blob/master/examples/rust/transfer-lamports/src/processor.rs

    im wondering if the try_borrow is in anchor?

    im confused about the difference between a PDA and an account that's owned by a program???
    what is the difference


    an account is owned by the system program by default. the system program has basic functions for creating more accounts 
    and sending lamports and such and such. acocunts can be assigned a new owner once and only once. this owner is 
    always a program, which has permission to deduct lamports from the account and modify its data however it wishes


    */

    #[access_control(not_burned(&ctx.accounts.check))]
    pub fn cash_check(ctx: Context<CashCheck>) -> Result<()> {
        let seeds = &[
            ctx.accounts.check.to_account_info().key.as_ref(),
            &[ctx.accounts.check.nonce],
        ];
        let signer = &[&seeds[..]];
        //this one is a bit more straightforward. 
        //we are doing a transfer from the vault, to the "to" account passed in
        //the check_signer has authority over the vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.check.amount)?;
        // Burn the check for one time use.
        ctx.accounts.check.burned = true;
        Ok(())
    }

    #[access_control(not_burned(&ctx.accounts.check))]
    pub fn cancel_check(ctx: Context<CancelCheck>) -> Result<()> {
        let seeds = &[
            ctx.accounts.check.to_account_info().key.as_ref(),
            &[ctx.accounts.check.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.from.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.check.amount)?;
        ctx.accounts.check.burned = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateCheck<'info> {
    // Check being created.
    #[account(zero)]
    check: Account<'info, Check>,
    // Check's token vault. it must be owned by the check_signer
    #[account(mut, constraint = &vault.owner == check_signer.key)]
    vault: Account<'info, TokenAccount>,
    // Program derived address for the check.
    check_signer: AccountInfo<'info>,
    // Token account the check is made from.
    #[account(mut, has_one = owner)]
    from: Account<'info, TokenAccount>,
    // Token account the check is made to.
    #[account(constraint = from.mint == to.mint)]
    to: Account<'info, TokenAccount>,
    // Owner of the `from` token account.
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

impl<'info> CreateCheck<'info> {
    pub fn accounts(ctx: &Context<CreateCheck>, nonce: u8) -> Result<()> {
        let signer = Pubkey::create_program_address(
            &[ctx.accounts.check.to_account_info().key.as_ref(), &[nonce]],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidCheckNonce)?;
        if &signer != ctx.accounts.check_signer.to_account_info().key {
            return Err(ErrorCode::InvalidCheckSigner.into());
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CashCheck<'info> {
    #[account(mut, has_one = vault, has_one = to)]
    check: Account<'info, Check>,
    #[account(mut)]
    vault: AccountInfo<'info>,
    #[account(
        seeds = [check.to_account_info().key.as_ref()],
        bump = check.nonce,
    )]
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    to: Account<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelCheck<'info> {
    #[account(mut, has_one = vault, has_one = from)]
    check: Account<'info, Check>,
    #[account(mut)]
    vault: AccountInfo<'info>,
    #[account(
        seeds = [check.to_account_info().key.as_ref()],
        bump = check.nonce,
    )]
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    from: Account<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[account]
pub struct Check {
    from: Pubkey,
    to: Pubkey,
    amount: u64,
    memo: Option<String>,
    vault: Pubkey,
    nonce: u8,
    burned: bool,
}

#[error]
pub enum ErrorCode {
    #[msg("The given nonce does not create a valid program derived address.")]
    InvalidCheckNonce,
    #[msg("The derived check signer does not match that which was given.")]
    InvalidCheckSigner,
    #[msg("The given check has already been burned.")]
    AlreadyBurned,
}

fn not_burned(check: &Check) -> Result<()> {
    if check.burned {
        return Err(ErrorCode::AlreadyBurned.into());
    }
    Ok(())
}
