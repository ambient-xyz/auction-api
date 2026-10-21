use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CommitSmallAuctionSettlementV4Accounts<'a, T> {
    pub coordinator: &'a T,
    pub bundle_escrow: &'a T,
    pub config_policy: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for CommitSmallAuctionSettlementV4Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [coordinator, bundle_escrow, config_policy, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        Ok(Self {
            coordinator,
            bundle_escrow,
            config_policy,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for CommitSmallAuctionSettlementV4Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.coordinator)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.config_policy))
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct CommitSmallAuctionSettlementV4Args {
    pub auction_hash: [u8; 32],
}
