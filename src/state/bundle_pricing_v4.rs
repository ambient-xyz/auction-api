use super::{BundleJobPricingV4, MAX_BUNDLE_JOBS};
use bytemuck::{Pod, Zeroable};

const BUNDLE_PRICING_V4_DOMAIN_TEXT: &[u8] = b"ambient.bundle.pricing.v6";

const fn pad_domain(domain: &[u8]) -> [u8; 32] {
    let mut padded = [0u8; 32];
    let mut index = 0;

    while index < domain.len() {
        padded[index] = domain[index];
        index += 1;
    }

    padded
}

pub const BUNDLE_PRICING_V4_DOMAIN: [u8; 32] = pad_domain(BUNDLE_PRICING_V4_DOMAIN_TEXT);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct BundlePricingCommitmentV4Message {
    pub domain: [u8; 32],
    pub bundle_hash: [u8; 32],

    pub pricing_entry_count: u8,
    pub _reserved0: [u8; 7],

    pub pricing_entries: [BundleJobPricingV4; MAX_BUNDLE_JOBS],
}

impl BundlePricingCommitmentV4Message {
    pub fn new(bundle_hash: [u8; 32], pricing_entries: &[BundleJobPricingV4]) -> Option<Self> {
        if pricing_entries.is_empty() || pricing_entries.len() > MAX_BUNDLE_JOBS {
            return None;
        }

        let mut message = Self::zeroed();

        message.domain = BUNDLE_PRICING_V4_DOMAIN;
        message.bundle_hash = bundle_hash;
        message.pricing_entry_count = u8::try_from(pricing_entries.len()).ok()?;
        message.pricing_entries[..pricing_entries.len()].copy_from_slice(pricing_entries);

        Some(message)
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
