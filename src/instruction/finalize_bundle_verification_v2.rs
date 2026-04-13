use crate::error::AuctionError;
use crate::{InstructionAccounts, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT: &[u8] = b"ambient.bundle.verify.v2";

const fn pad_domain_to_32_bytes(domain: &[u8]) -> [u8; 32] {
    let mut padded = [0u8; 32];
    let mut index = 0;
    while index < domain.len() {
        padded[index] = domain[index];
        index += 1;
    }
    padded
}

pub const FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN: [u8; 32] =
    pad_domain_to_32_bytes(FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(into = "u8", try_from = "u8")
)]
#[repr(transparent)]
pub struct VerificationVerdictV2(u8);

#[allow(non_upper_case_globals)]
impl VerificationVerdictV2 {
    pub const Unset: Self = Self(0);
    pub const Verified: Self = Self(1);
    pub const Rejected: Self = Self(2);

    pub const fn into_u8(self) -> u8 {
        self.0
    }
}

impl Default for VerificationVerdictV2 {
    fn default() -> Self {
        Self::Unset
    }
}

impl From<VerificationVerdictV2> for u8 {
    fn from(value: VerificationVerdictV2) -> Self {
        value.0
    }
}

impl TryFrom<u8> for VerificationVerdictV2 {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unset),
            1 => Ok(Self::Verified),
            2 => Ok(Self::Rejected),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        VerificationVerdictV2, FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN,
        FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT,
    };

    #[test]
    fn verification_verdict_v2_round_trips_through_raw_values() {
        assert_eq!(
            VerificationVerdictV2::try_from(0),
            Ok(VerificationVerdictV2::Unset)
        );
        assert_eq!(
            VerificationVerdictV2::try_from(2),
            Ok(VerificationVerdictV2::Rejected)
        );
        assert_eq!(u8::from(VerificationVerdictV2::Verified), 1);
        assert_eq!(VerificationVerdictV2::try_from(9), Err(9));
    }

    #[test]
    fn verification_verdict_v2_matches_on_associated_constants() {
        let label = match VerificationVerdictV2::Verified {
            VerificationVerdictV2::Unset => "unset",
            VerificationVerdictV2::Verified => "verified",
            VerificationVerdictV2::Rejected => "rejected",
            _ => "invalid",
        };

        assert_eq!(label, "verified");
    }

    #[test]
    fn finalize_bundle_verification_v2_domain_is_zero_padded() {
        assert_eq!(
            &FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN
                [..FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT.len()],
            FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT,
        );
        assert!(FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN
            [FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT.len()..]
            .iter()
            .all(|byte| *byte == 0));
    }
}

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
