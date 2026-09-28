use crate::error::AuctionError;
use crate::{BundleVerificationDisputeV2Kind, InstructionAccounts};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct DisputeBundleVerificationV2Accounts<'a, T> {
    pub dispute_payer: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verification_dispute: &'a T,
    pub bond_refund_recipient: &'a T,
    pub config_policy: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for DisputeBundleVerificationV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [dispute_payer, bundle_escrow, bundle_verification_dispute, bond_refund_recipient, config_policy, system_program, ..] =
            accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            dispute_payer,
            bundle_escrow,
            bundle_verification_dispute,
            bond_refund_recipient,
            config_policy,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for DisputeBundleVerificationV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.dispute_payer)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.bundle_verification_dispute))
            .chain(std::iter::once(self.bond_refund_recipient))
            .chain(std::iter::once(self.config_policy))
            .chain(std::iter::once(self.system_program))
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct DisputeBundleVerificationV2Args {
    pub kind: BundleVerificationDisputeV2Kind,
    pub _reserved: [u8; 7],
}
