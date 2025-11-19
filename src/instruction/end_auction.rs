use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// EndAuction instruction
///
/// Ends placing or revealing bids on an [`Auction`] account.
///
/// # Account References:
///
/// 1. `[WRITE]` Auction account
/// 1. `[WRITE]` Bundle account
/// 2. `[WRITE]` Vote account
/// 3. `[SIGNER]` Vote authority
#[derive(Debug, Clone)]
#[repr(C)]
pub struct EndAuctionAccounts<'a, T> {
    pub auction: &'a T,
    pub bundle: &'a T,
    pub vote_account: &'a T,
    pub payer: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for EndAuctionAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [auction, bundle, vote_account, payer] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            auction,
            bundle,
            vote_account,
            payer,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for EndAuctionAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.auction)
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.vote_account))
            .chain(std::iter::once(self.payer))
    }
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct EndAuctionArgs {}
