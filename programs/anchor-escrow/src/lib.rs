use anchor_lang::prelude::*;
mod contexts;
use contexts::*;
mod states;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Burn};

declare_id!("3xKqWLGxeDKxP7ZtB5KHnSBGsVVpHy5UmZAPQjsocoNW");

#[program]
pub mod anchor_escrow {


    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .initialize_escrow(seed, &ctx.bumps, initializer_amount, taker_amount)?;
        ctx.accounts.deposit(initializer_amount)
    }
    pub fn cancel(ctx: Context<Cancel>, amount_to_withdraw: u64) -> Result<()> {
        ctx.accounts.partial_refund(amount_to_withdraw)?;
        Ok(())
    }

    pub fn exchange(ctx: Context<Exchange>, exchange_amount: u64) -> Result<()> {
        ctx.accounts.deposit(exchange_amount)?;
        ctx.accounts.withdraw_and_close_vault(exchange_amount)
    }

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

    pub fn burn_pt_and_yt(
        ctx: Context<BurnTokens>,
        pt_amount: u64,
        yt_amount: u64,
    ) -> Result<()> {
        // Burn PT tokens
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.pt_mint.to_account_info(),
                    from: ctx.accounts.pt_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pt_amount,
        )?;

        // Burn YT tokens
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.yt_mint.to_account_info(),
                    from: ctx.accounts.yt_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            yt_amount,
        )?;

        Ok(())
    }

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        ctx.accounts.vault.initialize(ctx.accounts.user.key())
    }

    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        ctx.accounts.vault.deposit(amount)
    }

    pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
        ctx.accounts.vault.withdraw(amount)
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

#[derive(Accounts)]
pub struct BurnTokens<'info> {
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

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = Vault::LEN,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositToVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault_token_account", vault.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault_token_account", vault.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}