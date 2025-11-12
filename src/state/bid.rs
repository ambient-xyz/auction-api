use crate::{instruction::IpAddr, Pubkey, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
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
    pub status: BidStatus,
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
            status: BidStatus::Concealed,
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
            status: BidStatus::Concealed,
            canonical_bump: 0,
            ip: Default::default(),
            port: 0,
            public_key: Default::default(),
            pad: [0u8; 2],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u64)]
pub enum BidStatus {
    Concealed = 0,
    Revealed = 1,
}

unsafe impl Pod for BidStatus {}
