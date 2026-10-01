use crate::{error::AuctionError, InstructionAccounts};
use bytemuck::{Pod, Zeroable};

pub struct SealBundlePricingAccounts<'a, T> {
    pub coordinator: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verifier_pages: &'a [T],
}

impl<'a, T> TryFrom<&'a [T]> for SealBundlePricingAccounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [coordinator, bundle_escrow, pages @ ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        if pages.is_empty() {
            return Err(AuctionError::NotEnoughAccounts);
        }

        Ok(Self {
            coordinator,
            bundle_escrow,
            bundle_verifier_pages: pages,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SealBundlePricingAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.coordinator)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(self.bundle_verifier_pages.iter())
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct SealBundlePricingArgs {
    pub _reserved: [u8; 8],
}
