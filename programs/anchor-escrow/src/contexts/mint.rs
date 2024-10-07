use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[program]
pub mod token_minter {
    use super::*;

    pub fn mint_pt_and_yt(
        ctx: Context<MintTokens>,
        pt_amount: u64,
        yt_amount: u64,
    ) -> Result<()> {
        // Mint PT tokens
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.pt_mint.to_account_info(),
                    to: ctx.accounts.pt_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pt_amount,
        )?;

        // Mint YT tokens
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.yt_mint.to_account_info(),
                    to: ctx.accounts.yt_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            yt_amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub pt_mint: Account<'info, Mint>,
    #[account(mut)]
    pub yt_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pt_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yt_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}