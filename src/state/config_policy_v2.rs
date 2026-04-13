use super::{AccountLayoutVersion, Pubkey};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const CONFIG_POLICY_V2_ADMIN_CAPACITY: usize = 8;
pub const CONFIG_POLICY_V2_SERVICE_CAPACITY: usize = 16;

pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_OPEN_ESCROW_ARGS_BYPASS: u64 = 1 << 0;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_COMMIT_OVERRIDE: u64 = 1 << 1;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_RESULT_POST_OVERRIDE: u64 = 1 << 2;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_FINALIZE_OVERRIDE: u64 = 1 << 3;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_PAGE_BACKED_FINALIZE_BYPASS: u64 = 1 << 4;

pub const CONFIG_POLICY_V2_BUNDLE_ESCROW_RESERVED_BYTES: usize = 64;
pub const CONFIG_POLICY_V2_BUNDLE_VERIFIER_PAGE_RESERVED_BYTES: usize = 64;

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct ConfigPolicyV2 {
    pub bump: u64,
    pub minimum_bundle_auction_pairs: u64,
    pub policy_flags: u64,
    pub default_bundle_escrow_version: u8,
    pub default_bundle_verifier_page_version: u8,
    pub _reserved0: [u8; 6],
    pub admin_authorities: [Pubkey; CONFIG_POLICY_V2_ADMIN_CAPACITY],
    pub service_authorities: [Pubkey; CONFIG_POLICY_V2_SERVICE_CAPACITY],
    pub reserved: [[u8; 32]; 24],
}

impl Default for ConfigPolicyV2 {
    fn default() -> Self {
        Self {
            bump: 0,
            minimum_bundle_auction_pairs: 2,
            policy_flags: 0,
            default_bundle_escrow_version: AccountLayoutVersion::V1 as u8,
            default_bundle_verifier_page_version: AccountLayoutVersion::V1 as u8,
            _reserved0: [0; 6],
            admin_authorities: [Pubkey::default(); CONFIG_POLICY_V2_ADMIN_CAPACITY],
            service_authorities: [Pubkey::default(); CONFIG_POLICY_V2_SERVICE_CAPACITY],
            reserved: [[0; 32]; 24],
        }
    }
}

impl ConfigPolicyV2 {
    pub const LEN: usize = std::mem::size_of::<ConfigPolicyV2>();

    pub fn default_bundle_escrow_layout_version(&self) -> Option<AccountLayoutVersion> {
        AccountLayoutVersion::try_from(self.default_bundle_escrow_version).ok()
    }

    pub fn default_bundle_verifier_page_layout_version(&self) -> Option<AccountLayoutVersion> {
        AccountLayoutVersion::try_from(self.default_bundle_verifier_page_version).ok()
    }
}
