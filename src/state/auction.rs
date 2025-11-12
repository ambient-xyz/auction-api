use super::Pubkey;
use crate::{constant::PUBKEY_BYTES, RequestTier};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

/// Reverse auction on a bundle of requests
#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub struct Auction {
    /// Context length tier type
    pub context_length_tier: RequestTier,
    /// Expiry duration tier type
    pub expiry_duration_tier: RequestTier,
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
    /// Current status of the auction
    pub status: AuctionStatus,
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
            context_length_tier,
            expiry_duration_tier,
            max_context_length,
            bid_commitment_amount,
            ..Default::default()
        }
    }
}
impl Default for Auction {
    fn default() -> Self {
        Self {
            context_length_tier: RequestTier::Eco,
            expiry_duration_tier: RequestTier::Eco,
            request_bundle: Default::default(),
            expiry_slot: 0,
            max_context_length: RequestTier::Eco.get_max_context_length_tokens(),
            lowest_bid_price: None,
            winning_bid_price: None,
            winning_bid: Default::default(),
            lowest_bid: Default::default(),
            status: AuctionStatus::Active,
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
/// unsafe impl Pod for AuctionStatus {}
///
/// unsafe impl Pod for AuctionStatus {}
/// The default state is Active.
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

unsafe impl Pod for AuctionStatus {}
