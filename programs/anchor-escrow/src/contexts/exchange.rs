use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        close_account, transfer_checked, CloseAccount, Mint, Token, TokenAccount, TransferChecked,
    },
};

use crate::states::Escrow;

#[derive(Accounts)]
#[instruction(amount_to_exchange: u64)]
pub struct Exchange<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub initializer: SystemAccount<'info>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker
    )]
    pub taker_ata_a: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker
    )]
    pub taker_ata_b: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = initializer
    )]
    pub initializer_ata_b: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = mint_b,
        constraint = taker_ata_b.amount >= amount_to_exchange,
        close = initializer,
        seeds=[b"state", escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Exchange<'info> {
    pub fn deposit(&mut self, amount_to_exchange: u64) -> Result<()> {
        require!(amount_to_exchange <= self.escrow.taker_amount, ErrorCode::InvalidExchangeAmount);
        
        transfer_checked(
            self.into_deposit_context(),
            amount_to_exchange,
            self.mint_b.decimals,
        )
    }

    pub fn withdraw_and_close_vault(&mut self, amount_to_exchange: u64) -> Result<()> {
        let withdraw_amount = amount_to_exchange.min(self.escrow.initializer_amount);
        require!(withdraw_amount > 0, ErrorCode::InvalidExchangeAmount);
        
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"state",
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        transfer_checked(
            self.into_withdraw_context().with_signer(&signer_seeds),
            withdraw_amount,
            self.mint_a.decimals,
        )?;

        // If the entire amount is withdrawn, close the account
        if withdraw_amount == self.escrow.initializer_amount {
            close_account(self.into_close_context().with_signer(&signer_seeds))?;
        }

        Ok(())
    }

    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.initializer_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_withdraw_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid exchange amount")]
    InvalidExchangeAmount,
    // ... (other error codes)
}