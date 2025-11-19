use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// RevealBid instruction
///
/// Creates a concealed [`bid`] account for an associated auction.
///
/// # Account References:
///
/// 0. `[SIGNER]` Bid authority
/// 1. `[WRITE]` Concealed bid account
/// 2. `[WRITE]` Auction account
/// 3. `[WRITE]` Bundle account
/// 5. `[WRITE]` Bid authority vote account
/// 6. `[Signer]` Vote authority
#[derive(Debug, Clone)]
#[repr(C)]
pub struct RevealBidAccounts<'a, T> {
    pub bid_authority: &'a T,
    pub bid: &'a T,
    pub auction: &'a T,
    pub bundle: &'a T,
    pub vote_account: &'a T,
    pub vote_authority: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for RevealBidAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bid_authority, bid, auction, bundle, vote_account, vote_authority] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            bid_authority,
            bid,
            auction,
            bundle,
            vote_account,
            vote_authority,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for RevealBidAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bid_authority)
            .chain(std::iter::once(self.bid))
            .chain(std::iter::once(self.auction))
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.vote_account))
            .chain(std::iter::once(self.vote_authority))
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
pub struct RevealBidArgs {
    /// The hashed proposed bid price for the auction
    pub price_per_output_token: u64,
    /// seed used to generate price hash
    pub price_hash_seed: [u8; 32],
}
