use crate::error;
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Pricing/sizing tier attached to requests, bundles and auctions.
///
/// This enum is deliberately **not** `Pod`. It appears in account state (`Auction`, `JobRequest`,
/// `RawBundleData`, `BundleRegistry`) and in instruction arguments (`InitBundleArgs`,
/// `CancelBundleArgs`), and both are parsed straight out of untrusted bytes with `bytemuck`. `Pod`
/// asserts that every bit pattern of the type is a valid value, but this `#[repr(u64)]` enum has
/// only five valid values out of 2^64 — and they are not contiguous, they are `0..=4` in the
/// declaration order `Eco, Small, Standard, Pro, Large` with discriminants `0, 3, 1, 2, 4`. A `Pod`
/// impl therefore allowed a caller to submit any other `u64` and make every subsequent `match`
/// undefined behaviour rather than merely wrong. [`RequestTierRaw`] is stored instead, and callers
/// validate through `TryFrom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum RequestTier {
    Eco = 0,
    Small = 3,
    Standard = 1,
    Pro = 2,
    Large = 4,
}

/// Wire representation of [`RequestTier`] inside account state and instruction arguments.
///
/// `#[repr(transparent)]` over a `u64` keeps the byte layout of every surrounding struct identical
/// to the previous `RequestTier` field while making the `Pod` impl sound, because every `u64` bit
/// pattern is a valid `u64`. An unrecognised discriminant is reported as
/// [`error::AuctionError::InvalidRequestTier`] instead of being executed as a tier that does not
/// exist.
#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Eq, Hash, Default, Pod)]
#[repr(transparent)]
pub struct RequestTierRaw(u64);

impl RequestTierRaw {
    /// Builds the raw form from a known-valid tier.
    pub fn new(value: RequestTier) -> Self {
        value.into()
    }

    /// The stored discriminant, valid or not. Useful for diagnostics on a corrupt account.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<RequestTier> for RequestTierRaw {
    fn from(value: RequestTier) -> Self {
        Self(value.into())
    }
}

impl TryFrom<RequestTierRaw> for RequestTier {
    type Error = error::AuctionError;

    fn try_from(value: RequestTierRaw) -> Result<Self, Self::Error> {
        Self::try_from(value.0).map_err(|_| error::AuctionError::InvalidRequestTier)
    }
}

#[cfg(feature = "serde")]
impl Serialize for RequestTierRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RequestTier::try_from(*self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for RequestTierRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RequestTier::deserialize(deserializer).map(RequestTierRaw::from)
    }
}

impl std::fmt::Display for RequestTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            RequestTier::Eco => "Eco",
            RequestTier::Small => "Small",
            RequestTier::Standard => "Standard",
            RequestTier::Pro => "Pro",
            RequestTier::Large => "Large",
        };
        write!(f, "{name}")
    }
}

impl RequestTier {
    pub const ALL: [RequestTier; 5] = [
        RequestTier::Eco,
        RequestTier::Small,
        RequestTier::Standard,
        RequestTier::Pro,
        RequestTier::Large,
    ];

    pub fn get_verifiers_per_auction(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
            RequestTier::Small => 3,
        }
    }

    pub fn get_bid_reveal_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
            RequestTier::Small => 3,
        }
    }

    pub fn get_active_auction_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
            RequestTier::Small => 3,
        }
    }

    pub fn get_bundle_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 25,
            RequestTier::Standard => 25,
            RequestTier::Pro => 25,
            RequestTier::Large => 25,
            RequestTier::Small => 25,
        }
    }

    /// The maximum number of requests per bundle
    pub fn get_request_per_bundle(&self) -> u64 {
        match self {
            RequestTier::Eco => 30,
            RequestTier::Small => 12,
            RequestTier::Standard => 5,
            RequestTier::Pro => 2,
            RequestTier::Large => 1,
        }
    }

    /// Maximum allowed context length (in tokens) per tier
    pub fn get_max_context_length_tokens(&self) -> u64 {
        match self {
            RequestTier::Eco => 2_000,
            RequestTier::Small => 16_000,
            RequestTier::Standard => 32_000,
            RequestTier::Pro => 64_000,
            RequestTier::Large => 202_752,
        }
    }
    /// TODO: this should be enforced
    pub fn get_job_submission_duration_slots(&self) -> u64 {
        match self {
            RequestTier::Eco => 155,
            RequestTier::Standard => 145,
            RequestTier::Small => 145,
            RequestTier::Pro => 135,
            RequestTier::Large => 125,
        }
    }
    pub fn get_bid_commitment_amount_multiplier(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 2,
            RequestTier::Small => 2,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
        }
    }
    pub fn get_auction_credits_multiplier(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 2,
            RequestTier::Small => 2,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
        }
    }

    pub fn production_default_v2_settlement_window_slots(&self) -> u64 {
        32
    }

    pub fn production_default_v2_result_window_slots(&self) -> u64 {
        32
    }

    pub fn production_default_v2_verification_window_slots(&self) -> u64 {
        32
    }

    pub fn production_default_v2_claim_window_slots(&self) -> u64 {
        32
    }

    pub fn get_v2_settlement_window_slots(&self) -> u64 {
        self.production_default_v2_settlement_window_slots()
    }

    pub fn get_v2_result_window_slots(&self) -> u64 {
        self.production_default_v2_result_window_slots()
    }

    pub fn get_v2_verification_window_slots(&self) -> u64 {
        self.production_default_v2_verification_window_slots()
    }

    pub fn get_v2_claim_window_slots(&self) -> u64 {
        self.production_default_v2_claim_window_slots()
    }

    pub fn context_tier_for_tokens(tokens: u64) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|tier| tokens <= tier.get_max_context_length_tokens())
            .copied()
    }
}
