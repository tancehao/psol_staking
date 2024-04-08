use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    associated_token::AssociatedToken,
    token::{Mint, MintTo, Token, TokenAccount, Transfer, Burn},
};
use std::{cmp::max};

declare_id!("63rQ9E1dCQrfAA5AQfNMtbYHF3mU29K3Vc3SuwQ1uMYz");

#[program]
pub mod anchor_stake {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // initalize all the accounts for the pool
        Ok(())
    }

    pub fn new_staker(ctx:Context<NewStaker>) -> Result<()> {
        Ok(())
    }

    pub fn stake(ctx: Context<Operation>, deposit_amount: u64) -> Result<()> {

        let reciept = &mut ctx.accounts.reciept;
        // record new staked add
        if reciept.is_valid == 0 {
            reciept.is_valid = 1;
            if reciept.created_ts == 0 {
                reciept.created_ts = ctx.accounts.clock.unix_timestamp;
            }
            reciept.amount_deposited += deposit_amount;
        }

        // transfer stsol token from sender -> PDA vault
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.sender_stsol.to_account_info(),
                to: ctx.accounts.vault_stsol.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            }
        );
        token::transfer(transfer_ctx, deposit_amount)?;

        // transfer psol X to sender
        let mint_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                to: ctx.accounts.sender_psol.to_account_info(),
                mint: ctx.accounts.psol.to_account_info(),
                authority: ctx.accounts.psol.to_account_info(),
            }
        );
        let bump = ctx.bumps.psol;
        let tokenx_key = ctx.accounts.stsol.key();
        let pda_sign = &[
            b"psol",
            tokenx_key.as_ref(),
            &[bump],
        ];
        token::mint_to(
            mint_ctx.with_signer(&[pda_sign]),
            deposit_amount
        )?;

        Ok(())
    }

    pub fn slash(ctx: Context<Slash>) -> Result<()> {
        let reciept = &mut ctx.accounts.reciept;
        if reciept.is_valid == 0 { // must have staked in order to remove
            return Err(ProgramError::InvalidAccountData.into())
        }
        let slash_amount = 5;
        let deposited_amount = reciept.amount_deposited;
        reciept.amount_deposited = max(0, deposited_amount - slash_amount);

        // burn psol
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.psol.to_account_info(),
                from: ctx.accounts.staker_psol.to_account_info(),
                authority: ctx.accounts.staker.to_account_info()
            }
        );
        token::burn(burn_ctx, slash_amount)?;

        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_stsol.to_account_info(),
                to: ctx.accounts.vault_slash_pool.to_account_info(),
                authority: ctx.accounts.vault_stsol.to_account_info(),
            }
        );
        let bump = ctx.bumps.vault_stsol;
        let tokenx_key = ctx.accounts.stsol.key();
        let pda_sign = &[
            b"vault_stsol",
            tokenx_key.as_ref(),
            &[bump],
        ];

        token::transfer(
            transfer_ctx.with_signer(&[pda_sign]),
            slash_amount
        )?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Operation>) -> Result<()> {

        // compute bonus for staking
        let reciept = &mut ctx.accounts.reciept;
        if reciept.is_valid == 0 { // must have staked in order to remove
            return Err(ProgramError::InvalidAccountData.into())
        }
        let deposited_amount = reciept.amount_deposited;
        let start_time = reciept.created_ts;
        let curr_time = ctx.accounts.clock.unix_timestamp;

        // ~1 reward per second (note: unix time isnt always perfect)
        let diff_time = curr_time - start_time;
        // compute burn amount after rewards for staking
        let burn_amount = max(0, deposited_amount - (diff_time as u64)/2);

        // reset reciept validity
        reciept.is_valid = 0;

        // remove psol from sender
        if burn_amount > 0 {
            let burn_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.psol.to_account_info(),
                    from: ctx.accounts.sender_psol.to_account_info(),
                    authority: ctx.accounts.sender.to_account_info()
                }
            );
            token::burn(burn_ctx, deposited_amount)?;
        }

        // send back the deposited tokens
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_stsol.to_account_info(),
                to: ctx.accounts.sender_stsol.to_account_info(),
                authority: ctx.accounts.vault_stsol.to_account_info(),
            }
        );
        let bump = ctx.bumps.vault_stsol;
        let tokenx_key = ctx.accounts.stsol.key();
        let pda_sign = &[
            b"vault_stsol",
            tokenx_key.as_ref(),
            &[bump],
        ];

        token::transfer(
            transfer_ctx.with_signer(&[pda_sign]),
            deposited_amount
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub stsol: Account<'info, Mint>,

    #[account(
    init,
    payer=payer,
    seeds=[b"psol", stsol.key().as_ref()],
    bump,
    mint::decimals = stsol.decimals,
    mint::authority = psol
    )]
    pub psol: Account<'info, Mint>,

    // account to hold stsol
    #[account(
    init,
    payer=payer,
    seeds=[b"vault_stsol", stsol.key().as_ref()],
    bump,
    token::mint = stsol,
    token::authority = vault_stsol
    )]
    pub vault_stsol: Account<'info, TokenAccount>,

    #[account(
    init,
    payer=payer,
    seeds=[b"vault_slash_pool", stsol.key().as_ref()],
    bump,
    token::mint = stsol,
    token::authority = vault_slash_pool
    )]
    pub vault_slash_pool: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,
    // accounts required to init a new mint
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct NewStaker<'info> {
    pub stsol: Account<'info, Mint>,

    #[account(init, payer=sender, seeds=[b"reciept", stsol.key().as_ref(), sender.key().as_ref()], bump, space=8+1+8+8)]
    pub reciept: Account<'info, Receipt>,
    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Operation<'info> {
    pub stsol: Account<'info, Mint>,
    #[account(mut, seeds=[b"psol", stsol.key().as_ref()], bump)]
    pub psol: Account<'info, Mint>, // mint of psol token

    #[account(mut, seeds=[b"vault_stsol", stsol.key().as_ref()], bump)]
    pub vault_stsol: Account<'info, TokenAccount>, // mint to hold token stsol
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub sender_stsol: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sender_psol: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    #[account(mut, seeds=[b"reciept", stsol.key().as_ref(), sender.key().as_ref()], bump)]
    pub reciept: Account<'info, Receipt>,
}

#[derive(Accounts)]
pub struct Slash<'info> {
    pub stsol: Account<'info, Mint>,
    #[account(mut, seeds=[b"psol", stsol.key().as_ref()], bump)]
    pub psol: Account<'info, Mint>, // mint of psol token

    #[account(mut, seeds=[b"vault_stsol", stsol.key().as_ref()], bump)]
    pub vault_stsol: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_slash_pool: Account<'info, TokenAccount>,

    #[account(mut, seeds=[b"reciept", stsol.key().as_ref(), staker.key().as_ref()], bump)]
    pub reciept: Account<'info, Receipt>,

    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub staker_psol: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)] // will be init to zeros
pub struct Receipt {
    pub is_valid: u8,
    pub created_ts: i64,
    pub amount_deposited: u64,
}

