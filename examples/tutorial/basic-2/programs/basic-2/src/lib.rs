use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod basic_2 {
    use super::*;

    pub fn create(ctx: Context<Create>, authority: Pubkey) -> ProgramResult {
        let counter = &mut ctx.accounts.counter;
        counter.authority = authority;
        counter.count = 0;
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> ProgramResult {
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer = user, space = 8 + 40)]
    pub counter: Account<'info, Counter>,
    #[account(signer)]
    pub user: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

//Checks the `target` field on the account matches the `target` field in the struct deriving `Accounts`. |
//Checks the given account signed the transaction. |
//it looks like Account is just for managing ownership and accountInfo is the information that's actually associated with the account
// so the Account struct checks for correct ownership and then deserializes the accounts data into an AccountInfo if the deserial works


// account info is the actual information in the account
//account is a generic type that includes implementations of serialize, deserialize, owner, clone
// owner is just a pubkey which is the owner of the account
// the 

//not sure how account and accountInfo relate to each other exactly. or why you can 


#[derive(Accounts)]
pub struct Increment<'info> {
    //counter.authority and authority.key must be the same
    #[account(mut, has_one = authority)]
    pub counter: Account<'info, Counter>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
}

//this is declaring the counter struct that's going to be passed in as an account 
//in the create/increement structs, which derive Accounts
//so this is a procedural macro (which is a type of trait), that basically just adds all the vanilla syntax for Counter to be a subclass of account
#[account]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}
