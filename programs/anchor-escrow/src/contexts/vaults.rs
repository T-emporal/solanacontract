use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

impl Vault {
    pub const LEN: usize = 8 + 32 + 8;

    pub fn initialize(&mut self, owner: Pubkey) -> Result<()> {
        self.owner = owner;
        self.balance = 0;
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.balance = self.balance.checked_add(amount).unwrap();
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        if amount > self.balance {
            return Err(ErrorCode::InsufficientFunds.into());
        }
        self.balance = self.balance.checked_sub(amount).unwrap();
        Ok(())
    }

}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for withdrawal")]
    InsufficientFunds,
}