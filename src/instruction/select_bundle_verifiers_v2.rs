use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SelectBundleVerifiersV2Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub bundle_verification_dispute: Option<&'a T>,
}

impl<'a, T> TryFrom<&'a [T]> for SelectBundleVerifiersV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, remaining_accounts @ ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            bundle_escrow,
            bundle_verification_dispute: remaining_accounts.first(),
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SelectBundleVerifiersV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow).chain(self.bundle_verification_dispute)
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct SelectBundleVerifiersV2Args {}
