use super::{
    layout::{AccountDiscriminator, AccountLayoutVersion, ParsedAccountLayout},
    Pubkey,
};
use crate::constant::PUBKEY_BYTES;
use crate::state::request_tier::RequestTier;
use crate::{MaybePubkey, VERIFIERS_PER_AUCTION};
use bytemuck::{offset_of, Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    mem,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    ptr,
};

#[derive(Default, Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct Verifiers {
    pub keys: [Pubkey; VERIFIERS_PER_AUCTION],
}

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RawBundleData {
    /// Current status of the bundle
    pub status: BundleStatus,
    /// Context length tier type
    pub context_length_tier: RequestTier,
    /// Expiry duration tier type
    pub expiry_duration_tier: RequestTier,
    /// The auction for this bundle.
    pub auction: MaybePubkey,
    /// Assigned verifiers for this bundle.
    pub verifiers: Verifiers,
    /// The slot after which the auction cannot receive any more bids and is considered ended.
    pub expiry_slot: u64,
    /// The maximum input tokens each request can have
    pub max_context_length: u64,
    /// Total number of requests contained in this bundle.
    pub requests_len: u64,
    /// The number of job requests that were successfully verified
    pub num_verified_requests: u64,
    /// limit how much time winning bidder can take to submit all jobs
    pub job_submission_duration: u64,
    /// Total amount commited by the requesters
    pub request_committed_amount: u64,
    /// Total input tokens in the requests
    pub total_input_tokens: u64,
    /// Maximum output tokens to be generated for the requests
    pub maximum_output_tokens: u64,
    /// Total output tokens generated for the requests
    pub output_tokens_generated: u64,
    /// the parent bundle key is bundle is derived from
    pub parent_bundle_key: Pubkey,
    /// The child bundle key to be derived from this bundle
    pub child_bundle_key: MaybePubkey,
    /// bump for this bundle account
    pub bump: u64,
    /// assuming child_bundle bump is not zero (possible but statistically improbable)
    pub child_bundle_bump: Option<NonZeroU64>,
    /// assuming auction bump is not zero (possible but statistically improbable)
    pub auction_bump: Option<NonZeroU64>,
    /// payer key for the bundle account creation
    pub payer: Pubkey,
    /// The clearing price from the concluded auction for this bundle.
    /// Denotes the payment rate (in lamports) per output token that the
    /// winning bidder will receive for fulfilling the bundle’s requests.
    pub price_per_output_token: Option<NonZeroU64>,
}

/// Compatibility alias for the legacy bundle payload.
pub type RequestBundle = RawBundleData;

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct BundleLayoutTrailerV1 {
    pub discriminator: AccountDiscriminator,
    pub version: AccountLayoutVersion,
    pub reserved: [u8; 6],
}

impl BundleLayoutTrailerV1 {
    pub const LEN: usize = std::mem::size_of::<BundleLayoutTrailerV1>();

    pub const fn new() -> Self {
        Self {
            discriminator: AccountDiscriminator::Bundle,
            version: AccountLayoutVersion::V1,
            reserved: [0; 6],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidBundleTransition {
    pub from: BundleStatus,
    pub to: BundleStatus,
}

impl InvalidBundleTransition {
    const fn new(from: BundleStatus, to: BundleStatus) -> Self {
        Self { from, to }
    }
}

#[derive(Debug)]
pub struct RawBundleRef<'a> {
    layout: ParsedAccountLayout,
    raw: &'a RawBundleData,
}

#[derive(Debug)]
pub struct RawBundleMut<'a> {
    layout: ParsedAccountLayout,
    raw: &'a mut RawBundleData,
}

#[derive(Debug)]
pub enum BundleDataRef<'a> {
    Active(&'a RawBundleData),
    Full(&'a RawBundleData),
    PendingVerification(&'a RawBundleData),
    Verified(&'a RawBundleData),
    BadJobOutput(&'a RawBundleData),
    Canceled(&'a RawBundleData),
}

#[derive(Debug)]
pub enum BundleDataMut<'a> {
    Active(&'a mut RawBundleData),
    Full(&'a mut RawBundleData),
    PendingVerification(&'a mut RawBundleData),
    Verified(&'a mut RawBundleData),
    BadJobOutput(&'a mut RawBundleData),
    Canceled(&'a mut RawBundleData),
}

pub const fn bundle_account_len(version: AccountLayoutVersion) -> usize {
    match version {
        AccountLayoutVersion::LegacyV0 => RawBundleData::LEGACY_LEN,
        AccountLayoutVersion::V1 => RawBundleData::LEGACY_LEN + BundleLayoutTrailerV1::LEN,
    }
}

pub fn parse_bundle_layout(bytes: &[u8]) -> Option<ParsedAccountLayout> {
    if bytes.len() == RawBundleData::LEGACY_LEN {
        return Some(ParsedAccountLayout::legacy_v0(AccountDiscriminator::Bundle));
    }

    if bytes.len() != bundle_account_len(AccountLayoutVersion::V1) {
        return None;
    }

    let trailer =
        bytemuck::try_from_bytes::<BundleLayoutTrailerV1>(&bytes[RawBundleData::LEGACY_LEN..])
            .ok()?;
    if trailer.discriminator != AccountDiscriminator::Bundle
        || trailer.version != AccountLayoutVersion::V1
    {
        return None;
    }

    Some(ParsedAccountLayout::new(
        AccountDiscriminator::Bundle,
        AccountLayoutVersion::V1,
    ))
}

impl RawBundleData {
    pub const LEGACY_LEN: usize = std::mem::size_of::<RawBundleData>();
    pub const LEN: usize = Self::LEGACY_LEN;

    pub fn new(
        payer: [u8; PUBKEY_BYTES],
        parent_bundle_key: [u8; PUBKEY_BYTES],
        bump: u64,
        current_slot: u64,
        context_length_tier: RequestTier,
        expiry_duration_tier: RequestTier,
    ) -> Self {
        RawBundleData {
            payer: payer.into(),
            parent_bundle_key: parent_bundle_key.into(),
            bump,
            expiry_slot: current_slot.saturating_add(expiry_duration_tier.get_bundle_duration()),
            context_length_tier,
            expiry_duration_tier,
            max_context_length: context_length_tier.get_max_context_length_tokens(),
            ..Default::default()
        }
    }

    pub fn from_bytes<A: AsRef<[u8]>>(bytes: &A) -> Option<&Self> {
        let raw = RawBundleRef::from_bytes(bytes.as_ref())?;
        Some(raw.into_raw())
    }

    pub fn add_request_record(
        &mut self,
        commited_amount: u64,
        input_tokens: u64,
        max_output_tokens: u64,
    ) {
        self.requests_len = self.requests_len.saturating_add(1);
        self.request_committed_amount = self
            .request_committed_amount
            .saturating_add(commited_amount);
        self.total_input_tokens = self.total_input_tokens.saturating_add(input_tokens);
        self.maximum_output_tokens = self.maximum_output_tokens.saturating_add(max_output_tokens);
    }

    pub fn is_expired(&self, slot: u64) -> bool {
        self.requests_len < self.context_length_tier.get_request_per_bundle()
            && self.expiry_slot <= slot
    }

    pub fn write_legacy_bytes(&self, bytes: &mut [u8]) -> bool {
        if bytes.len() < Self::LEGACY_LEN {
            return false;
        }

        bytes[..Self::LEGACY_LEN].copy_from_slice(bytemuck::bytes_of(self));
        true
    }

    pub fn cancel_bundle_from_bytes(bytes: &mut [u8]) -> bool {
        let offset = offset_of!(RawBundleData, status);
        write_field(bytes, offset, BundleStatus::Canceled)
    }

    pub fn is_expired_from_bytes(bytes: &[u8], slot: u64) -> Option<bool> {
        let requests_len = Self::read_requests_len_from_bytes(bytes)?;
        let expiry_slot = Self::read_expiry_slot_from_bytes(bytes)?;
        let context_len_tier = Self::read_context_len_tier_from_bytes(bytes)?;
        Some(requests_len < context_len_tier.get_request_per_bundle() && expiry_slot <= slot)
    }

    fn read_expiry_slot_from_bytes(bytes: &[u8]) -> Option<u64> {
        let offset = offset_of!(RawBundleData, expiry_slot);
        read_field(bytes, offset)
    }

    fn read_requests_len_from_bytes(bytes: &[u8]) -> Option<u64> {
        let offset = offset_of!(RawBundleData, requests_len);
        read_field(bytes, offset)
    }

    fn read_context_len_tier_from_bytes(bytes: &[u8]) -> Option<RequestTier> {
        let offset = offset_of!(RawBundleData, context_length_tier);
        read_field(bytes, offset)
    }
}

impl Default for RawBundleData {
    fn default() -> Self {
        Self {
            requests_len: 0,
            job_submission_duration: RequestTier::Eco.get_job_submission_duration_slots(),
            request_committed_amount: 0,
            total_input_tokens: 0,
            maximum_output_tokens: 0,
            output_tokens_generated: 0,
            parent_bundle_key: Default::default(),
            child_bundle_key: Default::default(),
            num_verified_requests: 0,
            context_length_tier: RequestTier::Eco,
            expiry_duration_tier: RequestTier::Eco,
            auction: Default::default(),
            verifiers: Default::default(),
            expiry_slot: 0,
            max_context_length: RequestTier::Eco.get_max_context_length_tokens(),
            status: BundleStatus::Active,
            bump: 0,
            child_bundle_bump: None,
            auction_bump: None,
            payer: Default::default(),
            price_per_output_token: None,
        }
    }
}

impl<'a> RawBundleRef<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Option<Self> {
        let layout = parse_bundle_layout(bytes)?;
        let raw =
            bytemuck::try_from_bytes::<RawBundleData>(&bytes[..RawBundleData::LEGACY_LEN]).ok()?;
        Some(Self { layout, raw })
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.layout
    }

    pub fn as_raw(&self) -> &RawBundleData {
        self.raw
    }

    pub fn into_raw(self) -> &'a RawBundleData {
        self.raw
    }

    pub fn state(&self) -> BundleDataRef<'_> {
        BundleDataRef::from_raw(self.raw)
    }
}

impl Deref for RawBundleRef<'_> {
    type Target = RawBundleData;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> RawBundleMut<'a> {
    pub fn from_bytes(bytes: &'a mut [u8]) -> Option<Self> {
        let layout = parse_bundle_layout(bytes)?;
        let raw =
            bytemuck::try_from_bytes_mut::<RawBundleData>(&mut bytes[..RawBundleData::LEGACY_LEN])
                .ok()?;
        Some(Self { layout, raw })
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.layout
    }

    pub fn as_raw(&self) -> &RawBundleData {
        self.raw
    }

    pub fn as_raw_mut(&mut self) -> &mut RawBundleData {
        self.raw
    }

    pub fn into_raw(self) -> &'a mut RawBundleData {
        self.raw
    }

    pub fn state(&self) -> BundleDataRef<'_> {
        BundleDataRef::from_raw(self.raw)
    }

    pub fn state_mut(&mut self) -> BundleDataMut<'_> {
        BundleDataMut::from_raw(self.raw)
    }

    pub fn mark_full(&mut self) -> Result<(), InvalidBundleTransition> {
        self.state_mut().mark_full().map(|_| ())
    }

    pub fn mark_canceled(&mut self) -> Result<(), InvalidBundleTransition> {
        self.state_mut().mark_canceled().map(|_| ())
    }

    pub fn mark_verified(&mut self) -> Result<(), InvalidBundleTransition> {
        self.state_mut().mark_verified().map(|_| ())
    }
}

impl Deref for RawBundleMut<'_> {
    type Target = RawBundleData;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl DerefMut for RawBundleMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw
    }
}

impl<'a> BundleDataRef<'a> {
    pub fn from_raw(raw: &'a RawBundleData) -> Self {
        match raw.status {
            BundleStatus::Active => Self::Active(raw),
            BundleStatus::Full => Self::Full(raw),
            BundleStatus::PendingVerification => Self::PendingVerification(raw),
            BundleStatus::Verified => Self::Verified(raw),
            BundleStatus::BadJobOutput => Self::BadJobOutput(raw),
            BundleStatus::Canceled => Self::Canceled(raw),
        }
    }

    pub fn as_raw(&self) -> &RawBundleData {
        match self {
            Self::Active(raw)
            | Self::Full(raw)
            | Self::PendingVerification(raw)
            | Self::Verified(raw)
            | Self::BadJobOutput(raw)
            | Self::Canceled(raw) => raw,
        }
    }

    pub fn status(&self) -> BundleStatus {
        self.as_raw().status
    }
}

impl<'a> BundleDataMut<'a> {
    pub fn from_raw(raw: &'a mut RawBundleData) -> Self {
        match raw.status {
            BundleStatus::Active => Self::Active(raw),
            BundleStatus::Full => Self::Full(raw),
            BundleStatus::PendingVerification => Self::PendingVerification(raw),
            BundleStatus::Verified => Self::Verified(raw),
            BundleStatus::BadJobOutput => Self::BadJobOutput(raw),
            BundleStatus::Canceled => Self::Canceled(raw),
        }
    }

    pub fn as_raw(&self) -> &RawBundleData {
        match self {
            Self::Active(raw)
            | Self::Full(raw)
            | Self::PendingVerification(raw)
            | Self::Verified(raw)
            | Self::BadJobOutput(raw)
            | Self::Canceled(raw) => raw,
        }
    }

    pub fn into_raw(self) -> &'a mut RawBundleData {
        match self {
            Self::Active(raw)
            | Self::Full(raw)
            | Self::PendingVerification(raw)
            | Self::Verified(raw)
            | Self::BadJobOutput(raw)
            | Self::Canceled(raw) => raw,
        }
    }

    pub fn status(&self) -> BundleStatus {
        self.as_raw().status
    }

    pub fn mark_full(self) -> Result<Self, InvalidBundleTransition> {
        match self {
            Self::Active(raw) => {
                raw.status = BundleStatus::Full;
                Ok(Self::Full(raw))
            }
            state => Err(InvalidBundleTransition::new(
                state.status(),
                BundleStatus::Full,
            )),
        }
    }

    pub fn mark_canceled(self) -> Result<Self, InvalidBundleTransition> {
        match self {
            Self::Active(raw) => {
                raw.status = BundleStatus::Canceled;
                Ok(Self::Canceled(raw))
            }
            Self::Full(raw) => {
                raw.status = BundleStatus::Canceled;
                Ok(Self::Canceled(raw))
            }
            state => Err(InvalidBundleTransition::new(
                state.status(),
                BundleStatus::Canceled,
            )),
        }
    }

    pub fn mark_verified(self) -> Result<Self, InvalidBundleTransition> {
        match self {
            Self::Full(raw) => {
                raw.status = BundleStatus::Verified;
                Ok(Self::Verified(raw))
            }
            state => Err(InvalidBundleTransition::new(
                state.status(),
                BundleStatus::Verified,
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum BundleStatus {
    /// The bundle is currently active and accepting requests.
    Active = 0,
    /// The bundle is filled, awaiting job output submission
    Full = 2,
    /// The auction job output is not validated yet
    PendingVerification = 3,
    /// The auction job output has been validated
    Verified = 4,
    /// The job output is invalid
    BadJobOutput = 5,
    /// The bundle has failed to conclude
    Canceled = 6,
}

unsafe impl Pod for BundleStatus {}

fn read_field<T: Pod>(bytes: &[u8], offset: usize) -> Option<T> {
    let end = offset + mem::size_of::<T>();
    if end > bytes.len() {
        return None;
    }
    let ptr = unsafe { bytes.as_ptr().add(offset) as *const T };
    Some(unsafe { ptr::read_unaligned(ptr) })
}

fn write_field<T: Pod>(bytes: &mut [u8], offset: usize, value: T) -> bool {
    let size = std::mem::size_of::<T>();
    if bytes.len() < offset + size {
        return false;
    }

    let slice = &mut bytes[offset..offset + size];
    slice.copy_from_slice(bytemuck::bytes_of(&value));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use memoffset::offset_of;

    // Verify that the memory layout of RawBundleData matches assumptions made
    // elsewhere in the system (e.g., raw writes or FFI). Changes that break
    // these tests may require updating dependent code.
    #[test]
    fn layout_offsets() {
        assert_eq!(offset_of!(RawBundleData, status), 0);
        assert_eq!(offset_of!(RawBundleData, context_length_tier), 8);
        assert_eq!(offset_of!(RawBundleData, expiry_duration_tier), 16);
        assert_eq!(offset_of!(RawBundleData, expiry_slot), 152);
        assert_eq!(offset_of!(RawBundleData, requests_len), 168);
    }

    #[test]
    fn layout_sizes() {
        assert_eq!(size_of::<RequestTier>(), 8);
        assert_eq!(size_of::<BundleStatus>(), 8);

        let request = RawBundleData::default();
        let _: BundleStatus = request.status;
        let _: RequestTier = request.context_length_tier;
        let _: RequestTier = request.expiry_duration_tier;
        let _: u64 = request.expiry_slot;
        let _: u64 = request.requests_len;
    }

    #[test]
    fn request_bundle_alias_matches_legacy_layout() {
        assert_eq!(RequestBundle::LEN, RawBundleData::LEGACY_LEN);
    }

    #[test]
    fn request_bundle_remains_legacy_pod() {
        let raw = RawBundleData {
            expiry_slot: 5,
            requests_len: 2,
            ..Default::default()
        };

        let decoded = bytemuck::try_from_bytes::<RequestBundle>(bytemuck::bytes_of(&raw)).unwrap();
        assert_eq!(decoded, &raw);
    }

    #[test]
    fn current_bundle_bytes_classify_as_legacy_v0() {
        let raw = RawBundleData::default();
        let layout = parse_bundle_layout(bytemuck::bytes_of(&raw)).unwrap();
        assert_eq!(
            layout,
            ParsedAccountLayout::legacy_v0(AccountDiscriminator::Bundle)
        );
        assert!(layout.is_legacy());
    }

    #[test]
    fn state_view_matches_bundle_status() {
        let mut active = RawBundleData::default();
        assert!(matches!(
            BundleDataRef::from_raw(&active),
            BundleDataRef::Active(_)
        ));

        active.status = BundleStatus::Full;
        assert!(matches!(
            BundleDataRef::from_raw(&active),
            BundleDataRef::Full(_)
        ));

        active.status = BundleStatus::PendingVerification;
        assert!(matches!(
            BundleDataRef::from_raw(&active),
            BundleDataRef::PendingVerification(_)
        ));

        active.status = BundleStatus::Verified;
        assert!(matches!(
            BundleDataRef::from_raw(&active),
            BundleDataRef::Verified(_)
        ));

        active.status = BundleStatus::BadJobOutput;
        assert!(matches!(
            BundleDataRef::from_raw(&active),
            BundleDataRef::BadJobOutput(_)
        ));

        active.status = BundleStatus::Canceled;
        assert!(matches!(
            BundleDataRef::from_raw(&active),
            BundleDataRef::Canceled(_)
        ));
    }

    #[test]
    fn transition_helpers_update_status() {
        let mut raw = RawBundleData::default();
        let raw = BundleDataMut::from_raw(&mut raw)
            .mark_full()
            .unwrap()
            .into_raw();
        assert_eq!(raw.status, BundleStatus::Full);

        let raw = BundleDataMut::from_raw(raw)
            .mark_verified()
            .unwrap()
            .into_raw();
        assert_eq!(raw.status, BundleStatus::Verified);
    }

    #[test]
    fn transition_helpers_reject_invalid_moves() {
        let mut raw = RawBundleData {
            status: BundleStatus::Verified,
            ..Default::default()
        };

        let err = BundleDataMut::from_raw(&mut raw)
            .mark_canceled()
            .unwrap_err();
        assert_eq!(
            err,
            InvalidBundleTransition {
                from: BundleStatus::Verified,
                to: BundleStatus::Canceled,
            }
        );
    }

    #[test]
    fn raw_bundle_views_deref_to_legacy_payload() {
        let raw = RawBundleData {
            requests_len: 3,
            max_context_length: 42,
            ..Default::default()
        };
        let bytes = bytemuck::bytes_of(&raw);

        let parsed = RawBundleRef::from_bytes(bytes).unwrap();
        assert_eq!(parsed.requests_len, 3);
        assert_eq!(parsed.max_context_length, 42);

        let mut mutable_bytes = bytes.to_vec();
        let mut parsed = RawBundleMut::from_bytes(&mut mutable_bytes).unwrap();
        parsed.requests_len = 9;

        let reparsed = RawBundleData::from_bytes(&mutable_bytes).unwrap();
        assert_eq!(reparsed.requests_len, 9);
    }

    #[test]
    fn raw_bundle_mut_mark_helpers_match_state_transitions() {
        let raw = RawBundleData::default();
        let mut bytes = bytemuck::bytes_of(&raw).to_vec();

        {
            let mut parsed = RawBundleMut::from_bytes(&mut bytes).unwrap();
            parsed.mark_full().unwrap();
            parsed.mark_verified().unwrap();
        }

        let reparsed = RawBundleData::from_bytes(&bytes).unwrap();
        assert_eq!(reparsed.status, BundleStatus::Verified);
    }

    #[test]
    fn write_legacy_bytes_preserves_v1_trailer() {
        let raw = RawBundleData {
            expiry_slot: 5,
            requests_len: 2,
            ..Default::default()
        };
        let trailer = BundleLayoutTrailerV1::new();
        let mut bytes = vec![0_u8; bundle_account_len(AccountLayoutVersion::V1)];
        bytes[RawBundleData::LEGACY_LEN..].copy_from_slice(bytemuck::bytes_of(&trailer));

        assert!(raw.write_legacy_bytes(&mut bytes));
        assert_eq!(
            &bytes[..RawBundleData::LEGACY_LEN],
            bytemuck::bytes_of(&raw)
        );
        assert_eq!(
            &bytes[RawBundleData::LEGACY_LEN..],
            bytemuck::bytes_of(&trailer)
        );
        assert_eq!(
            parse_bundle_layout(&bytes),
            Some(ParsedAccountLayout::new(
                AccountDiscriminator::Bundle,
                AccountLayoutVersion::V1
            ))
        );
    }

    #[test]
    fn legacy_helpers_support_v1_bundle_bytes() {
        let raw = RawBundleData {
            expiry_slot: 5,
            requests_len: 0,
            ..Default::default()
        };
        let mut bytes = vec![0_u8; bundle_account_len(AccountLayoutVersion::V1)];
        bytes[..RawBundleData::LEGACY_LEN].copy_from_slice(bytemuck::bytes_of(&raw));
        bytes[RawBundleData::LEGACY_LEN..]
            .copy_from_slice(bytemuck::bytes_of(&BundleLayoutTrailerV1::new()));

        assert_eq!(RawBundleData::is_expired_from_bytes(&bytes, 5), Some(true));
        assert!(RawBundleData::cancel_bundle_from_bytes(&mut bytes));

        let canceled =
            bytemuck::try_from_bytes::<RawBundleData>(&bytes[..RawBundleData::LEGACY_LEN]).unwrap();
        assert_eq!(canceled.status, BundleStatus::Canceled);
    }
}
