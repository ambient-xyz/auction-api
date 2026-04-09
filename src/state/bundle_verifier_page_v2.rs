use super::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, ParsedAccountLayout, Pubkey,
};
use crate::{VerificationVerdictV2, VERIFIERS_PER_AUCTION};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

pub const BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES: usize = 6;

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct BundleVerifierPageV2Entry {
    pub job_id: Pubkey,
    pub posted_output_tokens: u64,
    pub accepted_output_tokens: u64,
    pub assigned_verifiers_token_ranges: [u64; VERIFIERS_PER_AUCTION * 2],
    pub verifier_reward_tokens: [u64; VERIFIERS_PER_AUCTION],
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
    pub entries: [BundleVerifierPageV2Entry; BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES],
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
    pub const LEN: usize = AccountHeaderV1::LEN + Self::PAYLOAD_LEN;

    pub fn from_bytes(bytes: &[u8]) -> Option<BundleVerifierPageV2Ref<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes::<AccountHeaderV1>(header_bytes).ok()?;
        if header.layout()
            != Some(ParsedAccountLayout::new(
                AccountDiscriminator::BundleVerifierPageV2,
                AccountLayoutVersion::V1,
            ))
        {
            return None;
        }

        let raw = bytemuck::try_from_bytes::<RawBundleVerifierPageV2Data>(raw_bytes).ok()?;
        Some(BundleVerifierPageV2Ref { header, raw })
    }

    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<BundleVerifierPageV2Mut<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes_mut::<AccountHeaderV1>(header_bytes).ok()?;
        if header.layout()
            != Some(ParsedAccountLayout::new(
                AccountDiscriminator::BundleVerifierPageV2,
                AccountLayoutVersion::V1,
            ))
        {
            return None;
        }

        let raw = bytemuck::try_from_bytes_mut::<RawBundleVerifierPageV2Data>(raw_bytes).ok()?;
        Some(BundleVerifierPageV2Mut { header, raw })
    }

    pub fn write_v1_bytes(&self, bytes: &mut [u8]) -> bool {
        if bytes.len() != Self::LEN {
            return false;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        header_bytes.copy_from_slice(bytemuck::bytes_of(&AccountHeaderV1::new(
            AccountDiscriminator::BundleVerifierPageV2,
        )));
        raw_bytes.copy_from_slice(bytemuck::bytes_of(self));
        true
    }

    pub fn write_entries(
        &mut self,
        bundle_escrow: Pubkey,
        page_index: u16,
        entry_count: u16,
        entries: [BundleVerifierPageV2Entry; BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES],
    ) -> bool {
        if usize::from(entry_count) > BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES {
            return false;
        }

        self.bundle_escrow = bundle_escrow;
        self.page_index = page_index;
        self.entry_count = entry_count;
        self.entries = entries;
        true
    }
}
