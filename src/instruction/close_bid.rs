use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// CloseBid instruction
///
/// closes a [`Bid`] account.
/// Additionally, closes an associated [`Auction`] account if it is empty.
///
/// # Account References:
///
/// 0. `[WRITE, SIGNER]` Bid authority
/// 1. `[WRITE]` Bid account
/// 2. `[WRITE]` Auction payer account
/// 3. `[WRITE]` Auction account
/// 4. `[WRITE]` Bundle account
/// 5. `[WRITE]` Bidder vote account
/// 6. `[Signer]` Vote authority
/// 7. `[READ]` Vote program

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CloseBidAccounts<'a, T> {
    pub bid_authority: &'a T,
    pub bid: &'a T,
    pub auction_payer: &'a T,
    pub auction: &'a T,
    pub bundle: &'a T,
    pub vote_account: &'a T,
    pub vote_authority: &'a T,
    pub vote_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for CloseBidAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bid_authority, bid, auction_payer, auction, bundle, vote_account, vote_authority, vote_program, ..] =
            accounts
        else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            bid_authority,
            bid,
            auction_payer,
            auction,
            bundle,
            vote_account,
            vote_authority,
            vote_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for CloseBidAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bid_authority)
            .chain(std::iter::once(self.bid))
            .chain(std::iter::once(self.auction_payer))
            .chain(std::iter::once(self.auction))
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.vote_account))
            .chain(std::iter::once(self.vote_authority))
            .chain(std::iter::once(self.vote_program))
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
pub struct CloseBidArgs {}
