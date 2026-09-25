use crate::InstructionAccounts;
use crate::error::AuctionError;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct InitBundleVerifierPageV2Accounts<'a, T> {
    pub payer: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verifier_page: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for InitBundleVerifierPageV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, bundle_escrow, bundle_verifier_page, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            payer,
            bundle_escrow,
            bundle_verifier_page,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitBundleVerifierPageV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.bundle_verifier_page))
            .chain(std::iter::once(self.system_program))
    }
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InitBundleVerifierPageV2Args {
    pub bundle_verifier_page_lamports: u64,
    pub page_index: u16,
    pub _reserved: [u8; 6],
}
