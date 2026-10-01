use crate::{error::AuctionError, InstructionAccounts};
use crate::{BundleJobPricingV6, MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES};
use bytemuck::{Pod, Zeroable};

pub struct PostBundlePricingV6Accounts<'a, T> {
    pub coordinator: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verifier_page: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for PostBundlePricingV6Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [coordinator, bundle_escrow, bundle_verifier_page, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            coordinator,
            bundle_escrow,
            bundle_verifier_page,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for PostBundlePricingV6Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.coordinator)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.bundle_verifier_page))
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct PostBundlePricingV6Args {
    pub page_index: u16,
    pub pricing_entry_count: u8,
    pub _reserved: [u8; 5],
    pub pricing_entries: [BundleJobPricingV6; MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
}
