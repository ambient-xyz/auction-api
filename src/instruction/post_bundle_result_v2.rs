use crate::error::AuctionError;
use crate::state::{BundleVerifierPageV2Entry, BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES};
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PostBundleResultV2Accounts<'a, T> {
    pub authority: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verifier_page: Option<&'a T>,
}

impl<'a, T> TryFrom<&'a [T]> for PostBundleResultV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [authority, bundle_escrow, rest @ ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            authority,
            bundle_escrow,
            bundle_verifier_page: rest.first(),
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for PostBundleResultV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.authority)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(self.bundle_verifier_page.into_iter())
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
pub struct PostBundleResultV2Args {
    pub result_hash: [u8; 32],
    pub posted_output_tokens: u64,
    pub page_index: u16,
    pub page_entry_count: u16,
    pub _reserved: [u8; 4],
    pub page_entries: [BundleVerifierPageV2Entry; BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES],
}
