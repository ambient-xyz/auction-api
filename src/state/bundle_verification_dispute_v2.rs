use super::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, ParsedAccountLayout, Pubkey,
};
use crate::{VerificationVerdictV2, MAX_VERIFIERS_PER_AUCTION};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(into = "u8", try_from = "u8")
)]
#[repr(transparent)]
pub struct BundleVerificationDisputeV2Kind(u8);

#[allow(non_upper_case_globals)]
impl BundleVerificationDisputeV2Kind {
    pub const Unset: Self = Self(0);
    pub const MissedVerification: Self = Self(1);
    pub const PaidVerdictDispute: Self = Self(2);

    pub const fn into_u8(self) -> u8 {
        self.0
    }
}

impl Default for BundleVerificationDisputeV2Kind {
    fn default() -> Self {
        Self::Unset
    }
}

impl From<BundleVerificationDisputeV2Kind> for u8 {
    fn from(value: BundleVerificationDisputeV2Kind) -> Self {
        value.0
    }
}

impl TryFrom<u8> for BundleVerificationDisputeV2Kind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unset),
            1 => Ok(Self::MissedVerification),
            2 => Ok(Self::PaidVerdictDispute),
            _ => Err(value),
        }
    }
}

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RawBundleVerificationDisputeV2Data {
    pub bundle_escrow: Pubkey,
    pub kind: BundleVerificationDisputeV2Kind,
    pub original_verdict: VerificationVerdictV2,
    pub bump: u8,
    pub _reserved0: [u8; 5],
    pub replacement_verifiers: [Pubkey; MAX_VERIFIERS_PER_AUCTION],
    pub replacement_deadline_slot: u64,
    pub bond_lamports: u64,
    pub bond_refund_recipient: Pubkey,
}

pub type BundleVerificationDisputeV2 = RawBundleVerificationDisputeV2Data;

#[derive(Debug)]
pub struct BundleVerificationDisputeV2Ref<'a> {
    header: &'a AccountHeaderV1,
    raw: &'a RawBundleVerificationDisputeV2Data,
}

#[derive(Debug)]
pub struct BundleVerificationDisputeV2Mut<'a> {
    header: &'a mut AccountHeaderV1,
    raw: &'a mut RawBundleVerificationDisputeV2Data,
}

impl<'a> BundleVerificationDisputeV2Ref<'a> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }

    pub fn as_raw(&self) -> &RawBundleVerificationDisputeV2Data {
        self.raw
    }
}

impl Deref for BundleVerificationDisputeV2Ref<'_> {
    type Target = RawBundleVerificationDisputeV2Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> BundleVerificationDisputeV2Mut<'a> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }

    pub fn as_raw(&self) -> &RawBundleVerificationDisputeV2Data {
        self.raw
    }

    pub fn as_raw_mut(&mut self) -> &mut RawBundleVerificationDisputeV2Data {
        self.raw
    }
}

impl Deref for BundleVerificationDisputeV2Mut<'_> {
    type Target = RawBundleVerificationDisputeV2Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl DerefMut for BundleVerificationDisputeV2Mut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw
    }
}

impl RawBundleVerificationDisputeV2Data {
    pub const PAYLOAD_LEN: usize = std::mem::size_of::<RawBundleVerificationDisputeV2Data>();
    pub const LEN: usize = AccountHeaderV1::LEN + Self::PAYLOAD_LEN;

    pub fn from_bytes(bytes: &[u8]) -> Option<BundleVerificationDisputeV2Ref<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::BundleVerificationDisputeV2
            || layout.version != AccountLayoutVersion::V1
        {
            return None;
        }

        let raw = bytemuck::try_from_bytes::<RawBundleVerificationDisputeV2Data>(raw_bytes).ok()?;
        Some(BundleVerificationDisputeV2Ref { header, raw })
    }

    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<BundleVerificationDisputeV2Mut<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes_mut::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::BundleVerificationDisputeV2
            || layout.version != AccountLayoutVersion::V1
        {
            return None;
        }

        let raw =
            bytemuck::try_from_bytes_mut::<RawBundleVerificationDisputeV2Data>(raw_bytes).ok()?;
        Some(BundleVerificationDisputeV2Mut { header, raw })
    }

    pub fn write_bytes(&self, bytes: &mut [u8]) -> bool {
        if bytes.len() != Self::LEN {
            return false;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        header_bytes.copy_from_slice(bytemuck::bytes_of(&AccountHeaderV1::new(
            AccountDiscriminator::BundleVerificationDisputeV2,
        )));
        raw_bytes.copy_from_slice(bytemuck::bytes_of(self));
        true
    }
}

impl Default for RawBundleVerificationDisputeV2Data {
    fn default() -> Self {
        Self {
            bundle_escrow: Pubkey::default(),
            kind: BundleVerificationDisputeV2Kind::Unset,
            original_verdict: VerificationVerdictV2::Unset,
            bump: 0,
            _reserved0: [0; 5],
            replacement_verifiers: [Pubkey::default(); MAX_VERIFIERS_PER_AUCTION],
            replacement_deadline_slot: 0,
            bond_lamports: 0,
            bond_refund_recipient: Pubkey::default(),
        }
    }
}
