use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ClaimVerifierLstakeV2Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub verifier_vote_account: &'a T,
    pub vote_program: &'a T,
    pub vote_authority: &'a T,
    pub config_policy: &'a T,
    pub bundle_verifier_pages: &'a [T],
}

impl<'a, T> TryFrom<&'a [T]> for ClaimVerifierLstakeV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, verifier_vote_account, vote_program, vote_authority, config_policy, bundle_verifier_pages @ ..] =
            accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            bundle_escrow,
            verifier_vote_account,
            vote_program,
            vote_authority,
            config_policy,
            bundle_verifier_pages,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for ClaimVerifierLstakeV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow)
            .chain(std::iter::once(self.verifier_vote_account))
            .chain(std::iter::once(self.vote_program))
            .chain(std::iter::once(self.vote_authority))
            .chain(std::iter::once(self.config_policy))
            .chain(self.bundle_verifier_pages.iter())
    }

    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct ClaimVerifierLstakeV2Args {}
