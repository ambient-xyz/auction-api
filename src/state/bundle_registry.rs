use super::RequestTierRaw;
use crate::Pubkey;
use bytemuck::{Pod, Zeroable};

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[repr(C)]
pub struct BundleRegistry {
    /// Context length tier type. Validated raw wrapper; see [`RequestTierRaw`].
    pub context_length_tier: RequestTierRaw,
    /// Expiry duration tier type. Validated raw wrapper; see [`RequestTierRaw`].
    pub expiry_duration_tier: RequestTierRaw,
    /// The latest bundle for this tier.
    pub latest_bundle: Pubkey,
    pub payer: Pubkey,
    /// bump used to derive this account
    pub bump: u64,
}

impl BundleRegistry {
    pub const LEN: usize = std::mem::size_of::<BundleRegistry>();

    pub fn from_bytes<A: AsRef<[u8]>>(bytes: &A) -> Option<&Self> {
        bytemuck::try_from_bytes(bytes.as_ref()).ok()
    }
}
