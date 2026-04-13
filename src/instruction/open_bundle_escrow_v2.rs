use crate::error::AuctionError;
use crate::{InstructionAccounts, RequestTier, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct OpenBundleEscrowV2Accounts<'a, T> {
    pub payer: &'a T,
    pub bundle_escrow: &'a T,
    pub config_policy: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for OpenBundleEscrowV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, bundle_escrow, config_policy, system_program, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            payer,
            bundle_escrow,
            config_policy,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for OpenBundleEscrowV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.config_policy))
            .chain(std::iter::once(self.system_program))
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
pub struct OpenBundleEscrowV2Args {
    pub bundle_version: u32,
    pub _reserved0: [u8; 4],
    pub reward_tier: RequestTier,
    pub bundle_hash: [u8; 32],
    pub coordinator: [u8; PUBKEY_BYTES],
    pub requester_refund_recipient: [u8; PUBKEY_BYTES],
    pub total_input_tokens: u64,
    pub max_output_tokens: u64,
    pub escrow_lamports: u64,
    pub settlement_deadline_slot: u64,
    pub result_deadline_slot: u64,
    pub verification_deadline_slot: u64,
    pub claim_deadline_slot: u64,
}
