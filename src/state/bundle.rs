use super::{
    layout::{AccountDiscriminator, AccountLayoutVersion, ParsedAccountLayout},
    Pubkey,
};
use crate::constant::PUBKEY_BYTES;
use crate::state::request_tier::{RequestTier, RequestTierRaw};
use crate::{error, MaybePubkey, VERIFIERS_PER_AUCTION};
use bytemuck::{offset_of, Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    /// Current status of the bundle.
    ///
    /// Stored as the validated raw wrapper rather than [`BundleStatus`] itself: this account is
    /// deserialized with `bytemuck`, so the field type must accept every bit pattern. Read it with
    /// `BundleStatus::try_from(raw.status)`, or through [`BundleDataRef`]/[`BundleDataMut`], which
    /// validate once and then encode the status in the variant.
    pub status: BundleStatusRaw,
    /// Context length tier type. Validated raw wrapper; see [`RequestTierRaw`].
    pub context_length_tier: RequestTierRaw,
    /// Expiry duration tier type. Validated raw wrapper; see [`RequestTierRaw`].
    pub expiry_duration_tier: RequestTierRaw,
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
    pub discriminator: u8,
    pub version: u8,
    pub reserved: [u8; 6],
}

impl BundleLayoutTrailerV1 {
    pub const LEN: usize = std::mem::size_of::<BundleLayoutTrailerV1>();

    pub const fn new() -> Self {
        Self {
            discriminator: AccountDiscriminator::Bundle as u8,
            version: AccountLayoutVersion::V1 as u8,
            reserved: [0; 6],
        }
    }

    pub fn layout(&self) -> Option<ParsedAccountLayout> {
        let discriminator = AccountDiscriminator::try_from(self.discriminator).ok()?;
        let version = AccountLayoutVersion::try_from(self.version).ok()?;
        Some(ParsedAccountLayout::new(discriminator, version))
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
        AccountLayoutVersion::V1 | AccountLayoutVersion::V2 => {
            RawBundleData::LEGACY_LEN + BundleLayoutTrailerV1::LEN
        }
    }
}

pub fn parse_bundle_layout(bytes: &[u8]) -> Option<ParsedAccountLayout> {
    if bytes.len() < RawBundleData::LEGACY_LEN {
        return None;
    }

    if let Some(trailer_bytes) =
        bytes.get(RawBundleData::LEGACY_LEN..RawBundleData::LEGACY_LEN + BundleLayoutTrailerV1::LEN)
    {
        let trailer = bytemuck::try_from_bytes::<BundleLayoutTrailerV1>(trailer_bytes).ok()?;
        let layout = trailer.layout();
        if layout
            == Some(ParsedAccountLayout::new(
                AccountDiscriminator::Bundle,
                AccountLayoutVersion::V1,
            ))
        {
            return layout;
        }
    }

    Some(ParsedAccountLayout::legacy_v0(AccountDiscriminator::Bundle))
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
            context_length_tier: context_length_tier.into(),
            expiry_duration_tier: expiry_duration_tier.into(),
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
        // A tier discriminant this program never wrote has no request-per-bundle budget to compare
        // against. Treating such a bundle as not-expired is the conservative answer: expiry is the
        // precondition for cancelling and reclaiming, so a corrupt account cannot be swept by a
        // caller that only checked `is_expired`.
        let Ok(context_length_tier) = RequestTier::try_from(self.context_length_tier) else {
            return false;
        };
        self.requests_len < context_length_tier.get_request_per_bundle() && self.expiry_slot <= slot
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
        // The written bytes are identical to the previous `BundleStatus::Canceled`; only the Rust
        // type changed, so this remains a single little-endian `u64` store at the same offset.
        write_field(bytes, offset, BundleStatusRaw::new(BundleStatus::Canceled))
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
        // Read the raw `u64` first and validate it, so a tier discriminant this program never wrote
        // yields `None` instead of a nonexistent enum value.
        let raw: RequestTierRaw = read_field(bytes, offset)?;
        RequestTier::try_from(raw).ok()
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
            context_length_tier: RequestTierRaw::new(RequestTier::Eco),
            expiry_duration_tier: RequestTierRaw::new(RequestTier::Eco),
            auction: Default::default(),
            verifiers: Default::default(),
            expiry_slot: 0,
            max_context_length: RequestTier::Eco.get_max_context_length_tokens(),
            status: BundleStatusRaw::new(BundleStatus::Active),
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
    /// Classifies a bundle by its stored status.
    ///
    /// Fallible because the status is read from account bytes: a discriminant this program never
    /// wrote has no variant to map onto and is reported as
    /// [`error::AuctionError::InvalidBundleStatus`] rather than silently treated as `Active`. Once
    /// this succeeds the status is encoded in the variant, so no later read can be invalid.
    pub fn from_raw(raw: &'a RawBundleData) -> Result<Self, error::AuctionError> {
        Ok(match BundleStatus::try_from(raw.status)? {
            BundleStatus::Active => Self::Active(raw),
            BundleStatus::Full => Self::Full(raw),
            BundleStatus::PendingVerification => Self::PendingVerification(raw),
            BundleStatus::Verified => Self::Verified(raw),
            BundleStatus::BadJobOutput => Self::BadJobOutput(raw),
            BundleStatus::Canceled => Self::Canceled(raw),
        })
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

    /// The bundle status, taken from the variant rather than re-read from the account bytes, so it
    /// is infallible by construction.
    pub fn status(&self) -> BundleStatus {
        match self {
            Self::Active(_) => BundleStatus::Active,
            Self::Full(_) => BundleStatus::Full,
            Self::PendingVerification(_) => BundleStatus::PendingVerification,
            Self::Verified(_) => BundleStatus::Verified,
            Self::BadJobOutput(_) => BundleStatus::BadJobOutput,
            Self::Canceled(_) => BundleStatus::Canceled,
        }
    }
}

impl<'a> BundleDataMut<'a> {
    /// Classifies a bundle by its stored status. See [`BundleDataRef::from_raw`] for why this is
    /// fallible.
    pub fn from_raw(raw: &'a mut RawBundleData) -> Result<Self, error::AuctionError> {
        Ok(match BundleStatus::try_from(raw.status)? {
            BundleStatus::Active => Self::Active(raw),
            BundleStatus::Full => Self::Full(raw),
            BundleStatus::PendingVerification => Self::PendingVerification(raw),
            BundleStatus::Verified => Self::Verified(raw),
            BundleStatus::BadJobOutput => Self::BadJobOutput(raw),
            BundleStatus::Canceled => Self::Canceled(raw),
        })
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

    /// The bundle status, taken from the variant rather than re-read from the account bytes, so it
    /// is infallible by construction.
    pub fn status(&self) -> BundleStatus {
        match self {
            Self::Active(_) => BundleStatus::Active,
            Self::Full(_) => BundleStatus::Full,
            Self::PendingVerification(_) => BundleStatus::PendingVerification,
            Self::Verified(_) => BundleStatus::Verified,
            Self::BadJobOutput(_) => BundleStatus::BadJobOutput,
            Self::Canceled(_) => BundleStatus::Canceled,
        }
    }

    pub fn mark_full(self) -> Result<Self, InvalidBundleTransition> {
        match self {
            Self::Active(raw) => {
                raw.status = BundleStatus::Full.into();
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
                raw.status = BundleStatus::Canceled.into();
                Ok(Self::Canceled(raw))
            }
            Self::Full(raw) => {
                raw.status = BundleStatus::Canceled.into();
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
                raw.status = BundleStatus::Verified.into();
                Ok(Self::Verified(raw))
            }
            state => Err(InvalidBundleTransition::new(
                state.status(),
                BundleStatus::Verified,
            )),
        }
    }
}

/// Lifecycle status of a request bundle.
///
/// This enum is deliberately **not** `Pod`. `RawBundleData` is deserialized from account data with
/// `bytemuck`, and `Pod` asserts that every bit pattern of the type is a valid value; this
/// `#[repr(u64)]` enum has six valid values out of 2^64, and its discriminants are not even
/// contiguous (there is no `1`), so a `Pod` impl would let corrupt or hostile account data
/// materialize a status that does not exist. The account stores [`BundleStatusRaw`].
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

/// Wire representation of [`BundleStatus`] inside account state.
///
/// `#[repr(transparent)]` over a `u64` keeps `RawBundleData`'s byte layout — and therefore
/// `RawBundleData::LEGACY_LEN` and every `offset_of!` in this module — identical to the previous
/// `status: BundleStatus` field, while making the `Pod` impl sound. An unknown discriminant is
/// reported as [`error::AuctionError::InvalidBundleStatus`].
#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default, Pod)]
#[repr(transparent)]
pub struct BundleStatusRaw(u64);

impl BundleStatusRaw {
    /// Builds the raw form from a known-valid status.
    pub fn new(value: BundleStatus) -> Self {
        value.into()
    }

    /// The stored discriminant, valid or not. Useful for diagnostics on a corrupt account.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<BundleStatus> for BundleStatusRaw {
    fn from(value: BundleStatus) -> Self {
        Self(value.into())
    }
}

impl TryFrom<BundleStatusRaw> for BundleStatus {
    type Error = error::AuctionError;

    fn try_from(value: BundleStatusRaw) -> Result<Self, Self::Error> {
        Self::try_from(value.0).map_err(|_| error::AuctionError::InvalidBundleStatus)
    }
}

#[cfg(feature = "serde")]
impl Serialize for BundleStatusRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        BundleStatus::try_from(*self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for BundleStatusRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BundleStatus::deserialize(deserializer).map(BundleStatusRaw::from)
    }
}

impl std::fmt::Display for BundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            BundleStatus::Active => "Active",
            BundleStatus::Full => "Full",
            BundleStatus::PendingVerification => "PendingVerification",
            BundleStatus::Verified => "Verified",
            BundleStatus::BadJobOutput => "BadJobOutput",
            BundleStatus::Canceled => "Canceled",
        };
        write!(f, "{name}")
    }
}

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
