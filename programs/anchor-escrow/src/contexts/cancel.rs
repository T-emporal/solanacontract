use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::states::Escrow;

#[derive(Accounts)]
#[instruction(amount_to_withdraw: u64)]
pub struct Cancel<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = initializer
    )]
    pub initializer_ata_a: Account<'info, TokenAccount>,
    #[account(
        mut,
        has_one = initializer,
        has_one = mint_a,
        seeds=[b"state", escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Cancel<'info> {
    pub fn partial_refund(&mut self, amount_to_withdraw: u64) -> Result<()> {
        require!(
            amount_to_withdraw <= self.escrow.initializer_amount,
            ErrorCode::InvalidWithdrawAmount
        );

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"state",
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        transfer_checked(
            self.into_refund_context().with_signer(&signer_seeds),
            amount_to_withdraw,
            self.mint_a.decimals,
        )?;

        self.escrow.initializer_amount = self.escrow.initializer_amount
            .checked_sub(amount_to_withdraw)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    fn into_refund_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.initializer_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid withdraw amount")]
    InvalidWithdrawAmount,
    #[msg("Arithmetic overflow")]
    Overflow,
}
