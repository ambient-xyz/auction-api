use crate::error::AuctionError;
use crate::{InstructionAccounts, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN: [u8; 32] =
    *b"ambient.bundle.verify.v2\0\0\0\0\0\0\0\0";

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FinalizeBundleVerificationV2Accounts<'a, T> {
    pub coordinator: &'a T,
    pub bundle_escrow: &'a T,
    pub winner_node: &'a T,
    pub requester_refund_recipient: &'a T,
    pub instructions_sysvar: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for FinalizeBundleVerificationV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [coordinator, bundle_escrow, winner_node, requester_refund_recipient, instructions_sysvar, ..] =
            accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            coordinator,
            bundle_escrow,
            winner_node,
            requester_refund_recipient,
            instructions_sysvar,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for FinalizeBundleVerificationV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.coordinator)
            .chain(std::iter::once(self.bundle_escrow))
            .chain(std::iter::once(self.winner_node))
            .chain(std::iter::once(self.requester_refund_recipient))
            .chain(std::iter::once(self.instructions_sysvar))
    }

    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u8)]
pub enum VerificationVerdictV2 {
    Unset = 0,
    Verified = 1,
    Rejected = 2,
}

unsafe impl Pod for VerificationVerdictV2 {}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct FinalizeBundleVerificationV2Args {
    pub verification_hash: [u8; 32],
    pub accepted_output_tokens: u64,
    pub verdict: VerificationVerdictV2,
    pub quorum_verifier_bitmap: u8,
    pub _reserved: [u8; 6],
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct FinalizeBundleVerificationV2Message {
    pub domain: [u8; 32],
    pub bundle_escrow: [u8; PUBKEY_BYTES],
    pub bundle_version: u32,
    pub _reserved0: [u8; 4],
    pub bundle_hash: [u8; 32],
    pub auction_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub verification_hash: [u8; 32],
    pub verdict: VerificationVerdictV2,
    pub _reserved1: [u8; 7],
    pub accepted_output_tokens: u64,
}

impl FinalizeBundleVerificationV2Message {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bundle_escrow: [u8; PUBKEY_BYTES],
        bundle_version: u32,
        bundle_hash: [u8; 32],
        auction_hash: [u8; 32],
        result_hash: [u8; 32],
        verification_hash: [u8; 32],
        verdict: VerificationVerdictV2,
        accepted_output_tokens: u64,
    ) -> Self {
        Self {
            domain: FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN,
            bundle_escrow,
            bundle_version,
            _reserved0: [0; 4],
            bundle_hash,
            auction_hash,
            result_hash,
            verification_hash,
            verdict,
            _reserved1: [0; 7],
            accepted_output_tokens,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}
