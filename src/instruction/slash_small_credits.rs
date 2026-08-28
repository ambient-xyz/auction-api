use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SlashSmallCreditsAccounts<'a, T> {
    pub slash_authority: &'a T,
    pub config_policy: &'a T,
    pub mint: &'a T,
    pub token_account: &'a T,
    pub token_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for SlashSmallCreditsAccounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [slash_authority, config_policy, mint, token_account, token_program, ..] = accounts
        else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            slash_authority,
            config_policy,
            mint,
            token_account,
            token_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SlashSmallCreditsAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        [
            self.slash_authority,
            self.config_policy,
            self.mint,
            self.token_account,
            self.token_program,
        ]
        .into_iter()
    }
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SlashSmallCreditsArgs {
    pub amount: u64,
    pub sequence: u64,
}
