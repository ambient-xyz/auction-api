use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ClaimWinnerLstakeV2Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub winner_vote_account: &'a T,
    pub vote_program: &'a T,
    pub vote_authority: &'a T,
    pub config_policy: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for ClaimWinnerLstakeV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, winner_vote_account, vote_program, vote_authority, config_policy, ..] =
            accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            bundle_escrow,
            winner_vote_account,
            vote_program,
            vote_authority,
            config_policy,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for ClaimWinnerLstakeV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow)
            .chain(std::iter::once(self.winner_vote_account))
            .chain(std::iter::once(self.vote_program))
            .chain(std::iter::once(self.vote_authority))
            .chain(std::iter::once(self.config_policy))
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct ClaimWinnerLstakeV2Args {}
