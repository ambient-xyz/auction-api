use super::{AccountData, RequestTier};
use crate::Pubkey;
use bytemuck::{CheckedBitPattern, NoUninit, Zeroable};

#[derive(Clone, Copy, Zeroable, NoUninit, CheckedBitPattern, Debug, PartialEq)]
#[repr(C)]
pub struct BundleRegistry {
    /// Context length tier type
    pub context_length_tier: RequestTier,
    /// Expiry duration tier type
    pub expiry_duration_tier: RequestTier,
    /// The latest bundle for this tier.
    pub latest_bundle: Pubkey,
    pub payer: Pubkey,
    /// bump used to derive this account
    pub bump: u64,
}

impl BundleRegistry {
    pub const LEN: usize = std::mem::size_of::<BundleRegistry>();

    pub fn from_bytes<A: AsRef<[u8]>>(bytes: &A) -> Option<&Self> {
        Self::try_from_bytes(bytes.as_ref()).ok()
    }
}
