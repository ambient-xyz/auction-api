use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CloseBundleVerifierPageV5Accounts<'a, T> {
    pub funder: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verifier_page: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for CloseBundleVerifierPageV5Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [funder, bundle_escrow, bundle_verifier_page, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            funder,
            bundle_escrow,
            bundle_verifier_page,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for CloseBundleVerifierPageV5Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.funder)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.bundle_verifier_page))
    }
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CloseBundleVerifierPageV5Args {
    pub page_index: u16,
}
