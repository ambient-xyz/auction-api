use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ClaimSmallCreditsV5Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub config_policy: &'a T,
    pub mint: &'a T,
    pub token_account: &'a T,
    pub token_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for ClaimSmallCreditsV5Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, config_policy, mint, token_account, token_program, ..] = accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        Ok(Self {
            bundle_escrow,
            config_policy,
            mint,
            token_account,
            token_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for ClaimSmallCreditsV5Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        [
            self.bundle_escrow,
            self.config_policy,
            self.mint,
            self.token_account,
            self.token_program,
        ]
        .into_iter()
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct ClaimSmallCreditsV5Args {}
