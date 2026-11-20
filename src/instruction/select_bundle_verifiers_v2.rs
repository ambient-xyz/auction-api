use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SelectBundleVerifiersV2Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub auction_verifiers: &'a T,
    pub slot_hashes: &'a T,
    /// Writable requester refund recipient for initial selection, or the dispute PDA for replacement.
    pub selection_account: Option<&'a T>,
    /// Writable stored bond refund recipient for paid replacement.
    pub bond_refund_recipient: Option<&'a T>,
}

impl<'a, T> TryFrom<&'a [T]> for SelectBundleVerifiersV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, auction_verifiers, slot_hashes, remaining_accounts @ ..] = accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        if remaining_accounts.len() > 2 {
            return Err(AuctionError::InvalidVerifierSelectionAccounts);
        }

        Ok(Self {
            bundle_escrow,
            auction_verifiers,
            slot_hashes,
            selection_account: remaining_accounts.first(),
            bond_refund_recipient: remaining_accounts.get(1),
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SelectBundleVerifiersV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow)
            .chain(std::iter::once(self.auction_verifiers))
            .chain(std::iter::once(self.slot_hashes))
            .chain(self.selection_account)
            .chain(self.bond_refund_recipient)
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct SelectBundleVerifiersV2Args {}
