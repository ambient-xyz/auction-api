use crate::MaybePubkey;
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Pod, Clone, Copy, Zeroable, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(C)]
pub struct Config {
    /// The update authority for the global configuration
    pub update_authority: MaybePubkey,
    pub minimum_bundle_auction_pairs: u64,
    pub canonical_bump: u64,
}

impl Config {
    pub const LEN: usize = std::mem::size_of::<Config>();
}
