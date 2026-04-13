use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExpireBundleEscrowV2Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub requester_refund_recipient: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for ExpireBundleEscrowV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, requester_refund_recipient, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            bundle_escrow,
            requester_refund_recipient,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for ExpireBundleEscrowV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow).chain(std::iter::once(self.requester_refund_recipient))
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
pub struct ExpireBundleEscrowV2Args {}
