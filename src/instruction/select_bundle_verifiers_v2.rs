use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SelectBundleVerifiersV2Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub auction_verifiers: &'a T,
    pub slot_hashes: &'a T,
    pub bundle_verification_dispute: Option<&'a T>,
}

impl<'a, T> TryFrom<&'a [T]> for SelectBundleVerifiersV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, auction_verifiers, slot_hashes, remaining_accounts @ ..] = accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        if remaining_accounts.len() > 1 {
            return Err(AuctionError::InvalidVerifierSelectionAccounts);
        }

        Ok(Self {
            bundle_escrow,
            auction_verifiers,
            slot_hashes,
            bundle_verification_dispute: remaining_accounts.first(),
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SelectBundleVerifiersV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow)
            .chain(std::iter::once(self.auction_verifiers))
            .chain(std::iter::once(self.slot_hashes))
            .chain(self.bundle_verification_dispute)
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct SelectBundleVerifiersV2Args {}
