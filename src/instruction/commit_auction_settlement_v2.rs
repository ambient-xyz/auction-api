use crate::error::AuctionError;
use crate::{InstructionAccounts, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CommitAuctionSettlementV2Accounts<'a, T> {
    pub coordinator: &'a T,
    pub bundle_escrow: &'a T,
    pub winner_vote_account: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for CommitAuctionSettlementV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [coordinator, bundle_escrow, winner_vote_account, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            coordinator,
            bundle_escrow,
            winner_vote_account,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for CommitAuctionSettlementV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.coordinator)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.winner_vote_account))
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
pub struct CommitAuctionSettlementV2Args {
    pub auction_hash: [u8; 32],
    pub winner_node_pubkey: [u8; PUBKEY_BYTES],
    pub clearing_price_per_output_token: u64,
}
