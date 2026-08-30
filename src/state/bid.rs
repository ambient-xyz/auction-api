use crate::{error, instruction::IpAddr, Pubkey, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::num::NonZeroU64;

#[derive(Pod, Clone, Copy, Zeroable, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(C)]
pub struct Bid {
    /// The bidding authority for this bid
    pub authority: Pubkey,
    /// The bid auction account ID
    pub auction: Pubkey,
    /// The hash of the bid price
    pub price_hash: [u8; 32],
    pub price_per_output_token: Option<NonZeroU64>,
    /// Whether the bid price is still concealed or has been revealed.
    ///
    /// Stored as the validated raw wrapper rather than [`BidStatus`] itself: `Bid` is deserialized
    /// from account data with `bytemuck`, so the field type must accept every bit pattern. Read it
    /// with `BidStatus::try_from(bid.status)`.
    pub status: BidStatusRaw,
    pub canonical_bump: u64,
    pub ip: IpAddr,
    pub port: u16,
    pub public_key: [u8; 32],
    pad: [u8; 2],
}

impl Bid {
    pub const LEN: usize = std::mem::size_of::<Bid>();

    pub fn new(
        authority: [u8; PUBKEY_BYTES],
        price_hash: [u8; 32],
        auction: [u8; PUBKEY_BYTES],
        canonical_bump: u64,
        ip: IpAddr,
        port: u16,
        public_key: [u8; 32],
    ) -> Self {
        Bid {
            authority: authority.into(),
            auction: auction.into(),
            price_hash,
            price_per_output_token: None,
            canonical_bump,
            ip,
            port,
            public_key,
            pad: Default::default(),
            status: BidStatusRaw::new(BidStatus::Concealed),
        }
    }

    pub fn from_bytes<A: AsRef<[u8]>>(bytes: &A) -> Option<&Self> {
        bytemuck::try_from_bytes(bytes.as_ref()).ok()
    }
}

impl Default for Bid {
    fn default() -> Self {
        Self {
            authority: Default::default(),
            auction: Default::default(),
            price_hash: Default::default(),
            price_per_output_token: None,
            status: BidStatusRaw::new(BidStatus::Concealed),
            canonical_bump: 0,
            ip: Default::default(),
            port: 0,
            public_key: Default::default(),
            pad: [0u8; 2],
        }
    }
}

/// Whether a bid's price is still concealed or has been revealed.
///
/// This enum is deliberately **not** `Pod`. `Bid` is deserialized from account data with `bytemuck`,
/// and `Pod` asserts that every bit pattern of the type is a valid value; a `#[repr(u64)]` enum with
/// two variants has two valid values out of 2^64. A `Pod` impl would therefore let corrupt or
/// hostile account data produce a discriminant that does not exist and make every later `match`
/// undefined behaviour. The account stores [`BidStatusRaw`] and callers convert through `TryFrom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u64)]
pub enum BidStatus {
    Concealed = 0,
    Revealed = 1,
}

/// Wire representation of [`BidStatus`] inside account state.
///
/// `#[repr(transparent)]` over a `u64` keeps `Bid`'s byte layout identical to the previous
/// `status: BidStatus` field while making the `Pod` impl sound, because every `u64` bit pattern is a
/// valid `u64`. An unknown discriminant is reported as
/// [`error::AuctionError::InvalidBidStatus`] instead of being executed as a nonexistent variant.
#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default, Pod)]
#[repr(transparent)]
pub struct BidStatusRaw(u64);

impl BidStatusRaw {
    /// Builds the raw form from a known-valid status.
    pub fn new(value: BidStatus) -> Self {
        value.into()
    }

    /// The stored discriminant, valid or not. Useful for diagnostics on a corrupt account.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<BidStatus> for BidStatusRaw {
    fn from(value: BidStatus) -> Self {
        Self(value.into())
    }
}

impl TryFrom<BidStatusRaw> for BidStatus {
    type Error = error::AuctionError;

    fn try_from(value: BidStatusRaw) -> Result<Self, Self::Error> {
        Self::try_from(value.0).map_err(|_| error::AuctionError::InvalidBidStatus)
    }
}

#[cfg(feature = "serde")]
impl Serialize for BidStatusRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        BidStatus::try_from(*self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for BidStatusRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BidStatus::deserialize(deserializer).map(BidStatusRaw::from)
    }
}

impl std::fmt::Display for BidStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            BidStatus::Concealed => "Concealed",
            BidStatus::Revealed => "Revealed",
        };
        write!(f, "{name}")
    }
}
