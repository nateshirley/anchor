use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

//so the basic setup here is that each of the pub functions are instruction handlers that you can call. like how 
//on the explorer it shows 4 instructions whatever

//each of the instructions have a context which must define all the accounts you are going to be using inside that instruction. 
//this is what allows the runtime to parallelize the instruction handling


#[program]
pub mod puppet {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        //i think what's happening here is that you have to pass it in to a function to initialize it 
        //so we're initializing the puppet account here, and then calling it from the other instruction
        Ok(())
    }

    pub fn set_data(ctx: Context<SetData>, data: u64) -> ProgramResult {
        let puppet = &mut ctx.accounts.puppet;
        puppet.data = data;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub puppet: Account<'info, Puppet>,
    #[account(signer)]
    pub user: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub puppet: Account<'info, Puppet>,
}

#[account]
pub struct Puppet {
    pub data: u64,
}
