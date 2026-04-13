use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// InitBundle instruction
///
/// Creates an [`AuctionVerifiers`] account.
///
/// # Account References:
///
/// 0. `[WRITE, SIGNER]` Funding account
/// 1. `[WRITE]` New auction verifiers account
/// 3. `[READ]` System program
#[derive(Debug, Clone)]
#[repr(C)]
pub struct InitAuctionVerifiersAccounts<'a, T> {
    pub payer: &'a T,
    pub auction_verifiers: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for InitAuctionVerifiersAccounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, auction_verifiers, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            payer,
            auction_verifiers,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitAuctionVerifiersAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.auction_verifiers))
            .chain(std::iter::once(self.system_program))
    }

    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Debug)]
#[repr(C)]
pub struct InitAuctionVerifiersArgs {}
