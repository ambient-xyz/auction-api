use super::Pubkey;
use crate::{
    constant::PUBKEY_BYTES,
    error,
    state::request_tier::{RequestTier, RequestTierRaw},
};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::num::NonZeroU64;

/// Reverse auction on a bundle of requests
#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub struct Auction {
    /// Context length tier type. Validated raw wrapper; see [`RequestTierRaw`].
    pub context_length_tier: RequestTierRaw,
    /// Expiry duration tier type. Validated raw wrapper; see [`RequestTierRaw`].
    pub expiry_duration_tier: RequestTierRaw,
    /// Bundle of requests for this auction.
    pub request_bundle: Pubkey,
    /// The slot after which the auction cannot receive any more bids and is considered ended.
    pub expiry_slot: u64,
    /// The maximum input tokens each request can have
    pub max_context_length: u64,
    /// The lowest bid price submitted
    pub lowest_bid_price: Option<NonZeroU64>,
    /// The second-lowest bid price submitted
    pub winning_bid_price: Option<NonZeroU64>,
    /// The public key of the winning bid account
    pub winning_bid: Pubkey,
    /// The public key of the lowest priced bid account
    pub lowest_bid: Pubkey,
    /// Current status of the auction.
    ///
    /// Stored as the validated raw wrapper rather than [`AuctionStatus`] itself: the account is
    /// deserialized with `bytemuck` straight out of chain data, so the field type must accept every
    /// bit pattern. Read it with `AuctionStatus::try_from(auction.status)`.
    pub status: AuctionStatusRaw,
    /// Total number of bids revealed
    pub bids_revealed: u64,
    /// Total number of concealed bids placed
    pub bids_placed: u64,
    /// Amount to be kept in each bid account as commitment,
    pub bid_commitment_amount: u64,
    /// Bump of the winning Bid account
    /// Assuming the bump is never zero
    pub winning_bid_bump: Option<NonZeroU64>,
    /// Bump of the lowest priced Bid account
    /// Assuming the bump is never zero
    pub lowest_bid_bump: Option<NonZeroU64>,
    /// bump for this auction account
    pub auction_bump: u64,
    /// The fee payer for creating this account
    pub payer: Pubkey,
}

impl Auction {
    pub const LEN: usize = std::mem::size_of::<Auction>();

    pub fn from_bytes<A: AsRef<[u8]>>(bytes: &A) -> Option<&Self> {
        bytemuck::try_from_bytes(bytes.as_ref()).ok()
    }
    #[allow(clippy::too_many_arguments)]
    pub fn init_from_auction(
        payer: [u8; PUBKEY_BYTES],
        request_bundle: [u8; PUBKEY_BYTES],
        auction_bump: u64,
        expiry_slot: u64,
        context_length_tier: RequestTier,
        expiry_duration_tier: RequestTier,
        max_context_length: u64,
        bid_commitment_amount: u64,
    ) -> Self {
        Auction {
            payer: payer.into(),
            request_bundle: request_bundle.into(),
            auction_bump,
            expiry_slot,
            context_length_tier: context_length_tier.into(),
            expiry_duration_tier: expiry_duration_tier.into(),
            max_context_length,
            bid_commitment_amount,
            ..Default::default()
        }
    }
}
impl Default for Auction {
    fn default() -> Self {
        Self {
            context_length_tier: RequestTierRaw::new(RequestTier::Eco),
            expiry_duration_tier: RequestTierRaw::new(RequestTier::Eco),
            request_bundle: Default::default(),
            expiry_slot: 0,
            max_context_length: RequestTier::Eco.get_max_context_length_tokens(),
            lowest_bid_price: None,
            winning_bid_price: None,
            winning_bid: Default::default(),
            lowest_bid: Default::default(),
            status: AuctionStatusRaw::new(AuctionStatus::Active),
            auction_bump: 0,
            winning_bid_bump: None,
            lowest_bid_bump: None,
            payer: Default::default(),
            bids_revealed: 0,
            bids_placed: 0,
            bid_commitment_amount: 0,
        }
    }
}

/// Represents the current status of an auction.
///
/// The default state is Active.
///
/// This enum is deliberately **not** `Pod`. `Auction` is deserialized from account data with
/// `bytemuck`, and `Pod` asserts that every bit pattern of the type is a valid value. A
/// `#[repr(u64)]` enum with four variants has four valid values out of 2^64, so a `Pod` impl on it
/// would let corrupt or hostile account data materialize a discriminant that does not exist and make
/// every later `match` undefined behaviour. The account stores [`AuctionStatusRaw`] instead and
/// callers convert through `TryFrom`, which is the same pattern `JobVerificationState` already uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum AuctionStatus {
    /// The auction is currently active and accepting encrypted bids.
    Active = 0,

    /// The bidding phase has ended; participants are now revealing their bids.
    RevealingBids = 1,

    /// The auction has concluded and the final price has been determined.
    Ended = 2,

    /// The auction is canceled; no bids.
    Canceled = 3,
}

/// Wire representation of [`AuctionStatus`] inside account state.
///
/// `#[repr(transparent)]` over a `u64` keeps the byte layout of `Auction` byte-for-byte identical to
/// the previous `status: AuctionStatus` field while making the `Pod` impl sound, because every `u64`
/// bit pattern is a valid `u64`. Validation moves from "assumed" to `TryFrom`, which reports an
/// unknown discriminant as [`error::AuctionError::InvalidAuctionStatus`].
#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default, Pod)]
#[repr(transparent)]
pub struct AuctionStatusRaw(u64);

impl AuctionStatusRaw {
    /// Builds the raw form from a known-valid status.
    pub fn new(value: AuctionStatus) -> Self {
        value.into()
    }

    /// The stored discriminant, valid or not. Useful for diagnostics on a corrupt account.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<AuctionStatus> for AuctionStatusRaw {
    fn from(value: AuctionStatus) -> Self {
        Self(value.into())
    }
}

impl TryFrom<AuctionStatusRaw> for AuctionStatus {
    type Error = error::AuctionError;

    fn try_from(value: AuctionStatusRaw) -> Result<Self, Self::Error> {
        Self::try_from(value.0).map_err(|_| error::AuctionError::InvalidAuctionStatus)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AuctionStatusRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AuctionStatus::try_from(*self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AuctionStatusRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AuctionStatus::deserialize(deserializer).map(AuctionStatusRaw::from)
    }
}

impl std::fmt::Display for AuctionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AuctionStatus::Active => "Active",
            AuctionStatus::RevealingBids => "RevealingBids",
            AuctionStatus::Ended => "Ended",
            AuctionStatus::Canceled => "Canceled",
        };
        write!(f, "{name}")
    }
}
