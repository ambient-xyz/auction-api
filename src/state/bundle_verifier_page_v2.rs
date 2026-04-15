use super::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, ParsedAccountLayout, Pubkey,
    CONFIG_POLICY_V2_BUNDLE_VERIFIER_PAGE_RESERVED_BYTES,
};
use crate::{MAX_VERIFIERS_PER_AUCTION, VerificationVerdictV2};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Protocol-level page entry capacity for `BundleVerifierPageV2`.
pub const MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES: usize = 6;
/// Compatibility alias for one release cycle. Prefer `MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES`.
pub const BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES: usize = MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES;

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct BundleVerifierPageV2Entry {
    pub job_id: Pubkey,
    pub posted_output_tokens: u64,
    pub accepted_output_tokens: u64,
    pub assigned_verifiers_token_ranges: [u64; MAX_VERIFIERS_PER_AUCTION * 2],
    pub verifier_reward_tokens: [u64; MAX_VERIFIERS_PER_AUCTION],
    pub verdict: VerificationVerdictV2,
    pub verifier_claimed_bitmap: u8,
    pub _reserved: [u8; 6],
}

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RawBundleVerifierPageV2Data {
    pub bundle_escrow: Pubkey,
    pub page_index: u16,
    pub entry_count: u16,
    pub _reserved0: [u8; 4],
    pub entries: [BundleVerifierPageV2Entry; MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
}

pub type BundleVerifierPageV2 = RawBundleVerifierPageV2Data;

#[derive(Debug)]
pub struct BundleVerifierPageV2Ref<'a> {
    header: &'a AccountHeaderV1,
    raw: &'a RawBundleVerifierPageV2Data,
}

#[derive(Debug)]
pub struct BundleVerifierPageV2Mut<'a> {
    header: &'a mut AccountHeaderV1,
    raw: &'a mut RawBundleVerifierPageV2Data,
}

impl<'a> BundleVerifierPageV2Ref<'a> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }

    pub fn as_raw(&self) -> &RawBundleVerifierPageV2Data {
        self.raw
    }
}

impl Deref for BundleVerifierPageV2Ref<'_> {
    type Target = RawBundleVerifierPageV2Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> BundleVerifierPageV2Mut<'a> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }

    pub fn as_raw(&self) -> &RawBundleVerifierPageV2Data {
        self.raw
    }

    pub fn as_raw_mut(&mut self) -> &mut RawBundleVerifierPageV2Data {
        self.raw
    }
}

impl Deref for BundleVerifierPageV2Mut<'_> {
    type Target = RawBundleVerifierPageV2Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl DerefMut for BundleVerifierPageV2Mut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw
    }
}

impl RawBundleVerifierPageV2Data {
    pub const PAYLOAD_LEN: usize = std::mem::size_of::<RawBundleVerifierPageV2Data>();
    pub const LEN: usize = Self::LEN_V1;
    pub const LEN_V1: usize = AccountHeaderV1::LEN + Self::PAYLOAD_LEN;
    pub const LEN_V2: usize = AccountHeaderV1::LEN
        + Self::PAYLOAD_LEN
        + CONFIG_POLICY_V2_BUNDLE_VERIFIER_PAGE_RESERVED_BYTES;

    pub const fn account_len(version: AccountLayoutVersion) -> usize {
        match version {
            AccountLayoutVersion::V1 => Self::LEN_V1,
            AccountLayoutVersion::V2 => Self::LEN_V2,
            AccountLayoutVersion::LegacyV0 => 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<BundleVerifierPageV2Ref<'_>> {
        if bytes.len() < AccountHeaderV1::LEN + Self::PAYLOAD_LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::BundleVerifierPageV2 {
            return None;
        }
        let expected_len = Self::account_len(layout.version);
        if expected_len == 0 || bytes.len() != expected_len {
            return None;
        }

        let (raw_bytes, _reserved) = raw_bytes.split_at(Self::PAYLOAD_LEN);
        let raw = bytemuck::try_from_bytes::<RawBundleVerifierPageV2Data>(raw_bytes).ok()?;
        Some(BundleVerifierPageV2Ref { header, raw })
    }

    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<BundleVerifierPageV2Mut<'_>> {
        let bytes_len = bytes.len();
        if bytes_len < AccountHeaderV1::LEN + Self::PAYLOAD_LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes_mut::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::BundleVerifierPageV2 {
            return None;
        }
        let expected_len = Self::account_len(layout.version);
        if expected_len == 0 || bytes_len != expected_len {
            return None;
        }

        let (raw_bytes, _reserved) = raw_bytes.split_at_mut(Self::PAYLOAD_LEN);
        let raw = bytemuck::try_from_bytes_mut::<RawBundleVerifierPageV2Data>(raw_bytes).ok()?;
        Some(BundleVerifierPageV2Mut { header, raw })
    }

    pub fn read(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes).map(|account| *account.as_raw())
    }

    pub fn write_v1_bytes(&self, bytes: &mut [u8]) -> bool {
        self.write_bytes_with_layout(bytes, AccountLayoutVersion::V1)
    }

    pub fn write_v2_bytes(&self, bytes: &mut [u8]) -> bool {
        self.write_bytes_with_layout(bytes, AccountLayoutVersion::V2)
    }

    pub fn write_bytes_with_layout(&self, bytes: &mut [u8], version: AccountLayoutVersion) -> bool {
        let expected_len = Self::account_len(version);
        if expected_len == 0 || bytes.len() != expected_len {
            return false;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        header_bytes.copy_from_slice(bytemuck::bytes_of(&AccountHeaderV1 {
            discriminator: AccountDiscriminator::BundleVerifierPageV2 as u8,
            version: version as u8,
            reserved: [0; 6],
        }));
        let (raw_bytes, reserved_bytes) = raw_bytes.split_at_mut(Self::PAYLOAD_LEN);
        raw_bytes.copy_from_slice(bytemuck::bytes_of(self));
        reserved_bytes.fill(0);
        true
    }

    pub fn write_entries(
        &mut self,
        bundle_escrow: Pubkey,
        page_index: u16,
        entry_count: u16,
        entries: [BundleVerifierPageV2Entry; MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
    ) -> bool {
        if usize::from(entry_count) > MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES {
            return false;
        }

        self.bundle_escrow = bundle_escrow;
        self.page_index = page_index;
        self.entry_count = entry_count;
        self.entries = entries;
        true
    }
}
