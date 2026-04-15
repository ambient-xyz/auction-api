use super::{AccountLayoutVersion, Pubkey, RequestTier};
use crate::{V2_VERIFIER_QUORUM, VERIFIERS_PER_AUCTION};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const CONFIG_POLICY_V2_ADMIN_CAPACITY: usize = 8;
pub const CONFIG_POLICY_V2_SERVICE_CAPACITY: usize = 16;
pub const CONFIG_POLICY_V2_TIER_CONFIG_COUNT: usize = 5;

pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_OPEN_ESCROW_ARGS_BYPASS: u64 = 1 << 0;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_COMMIT_OVERRIDE: u64 = 1 << 1;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_RESULT_POST_OVERRIDE: u64 = 1 << 2;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_FINALIZE_OVERRIDE: u64 = 1 << 3;
pub const CONFIG_POLICY_V2_FLAG_ALLOW_SERVICE_PAGE_BACKED_FINALIZE_BYPASS: u64 = 1 << 4;

pub const CONFIG_POLICY_V2_BUNDLE_ESCROW_RESERVED_BYTES: usize = 64;
pub const CONFIG_POLICY_V2_BUNDLE_VERIFIER_PAGE_RESERVED_BYTES: usize = 64;
pub const CONFIG_POLICY_V2_TYPED_RESERVED_WORDS: usize = 8;
pub const CONFIG_POLICY_V2_TYPED_RESERVED_LAYOUT_PADDING_BYTES: usize = 7;
pub const CONFIG_POLICY_V2_TYPED_RESERVED_TAIL_BYTES: usize = 16;

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RequestTierConfigV2 {
    pub bid_reveal_duration: u64,
    pub active_auction_duration: u64,
    pub bundle_duration: u64,
    pub requests_per_bundle: u64,
    pub max_context_length_tokens: u64,
    pub job_submission_duration_slots: u64,
    pub bid_commitment_amount_multiplier: u64,
    pub auction_credits_multiplier: u64,
    pub settlement_window_slots: u64,
    pub result_window_slots: u64,
    pub verification_window_slots: u64,
    pub claim_window_slots: u64,
}

impl RequestTierConfigV2 {
    pub fn from_request_tier(tier: RequestTier) -> Self {
        Self {
            bid_reveal_duration: tier.get_bid_reveal_duration(),
            active_auction_duration: tier.get_active_auction_duration(),
            bundle_duration: tier.get_bundle_duration(),
            requests_per_bundle: tier.get_request_per_bundle(),
            max_context_length_tokens: tier.get_max_context_length_tokens(),
            job_submission_duration_slots: tier.get_job_submission_duration_slots(),
            bid_commitment_amount_multiplier: tier.get_bid_commitment_amount_multiplier(),
            auction_credits_multiplier: tier.get_auction_credits_multiplier(),
            settlement_window_slots: tier.get_v2_settlement_window_slots(),
            result_window_slots: tier.get_v2_result_window_slots(),
            verification_window_slots: tier.get_v2_verification_window_slots(),
            claim_window_slots: tier.get_v2_claim_window_slots(),
        }
    }

    pub fn validate(&self) -> bool {
        self.bid_reveal_duration != 0
            && self.active_auction_duration != 0
            && self.bundle_duration != 0
            && self.requests_per_bundle != 0
            && self.max_context_length_tokens != 0
            && self.job_submission_duration_slots != 0
            && self.bid_commitment_amount_multiplier != 0
            && self.auction_credits_multiplier != 0
            && self.settlement_window_slots != 0
            && self.result_window_slots != 0
            && self.verification_window_slots != 0
            && self.claim_window_slots != 0
    }

    pub fn settlement_window_slots(&self, tier: RequestTier) -> u64 {
        if self.settlement_window_slots == 0 {
            tier.get_v2_settlement_window_slots()
        } else {
            self.settlement_window_slots
        }
    }

    pub fn result_window_slots(&self, tier: RequestTier) -> u64 {
        if self.result_window_slots == 0 {
            tier.get_v2_result_window_slots()
        } else {
            self.result_window_slots
        }
    }

    pub fn verification_window_slots(&self, tier: RequestTier) -> u64 {
        if self.verification_window_slots == 0 {
            tier.get_v2_verification_window_slots()
        } else {
            self.verification_window_slots
        }
    }

    pub fn claim_window_slots(&self, tier: RequestTier) -> u64 {
        if self.claim_window_slots == 0 {
            tier.get_v2_claim_window_slots()
        } else {
            self.claim_window_slots
        }
    }
}

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct ConfigPolicyV2 {
    pub bump: u64,
    pub minimum_bundle_auction_pairs: u64,
    pub policy_flags: u64,
    pub _reserved0: [u8; 8],
    pub admin_authorities: [Pubkey; CONFIG_POLICY_V2_ADMIN_CAPACITY],
    pub service_authorities: [Pubkey; CONFIG_POLICY_V2_SERVICE_CAPACITY],
    pub v2_verifiers_per_auction: u8,
    pub v2_verifier_quorum: u8,
    pub _reserved1: [u8; 6],
    pub tier_configs: [RequestTierConfigV2; CONFIG_POLICY_V2_TIER_CONFIG_COUNT],
    pub reserved_words: [[u8; 32]; CONFIG_POLICY_V2_TYPED_RESERVED_WORDS],
    pub v2_account_layout_version: u8,
    pub _reserved2: [u8; CONFIG_POLICY_V2_TYPED_RESERVED_LAYOUT_PADDING_BYTES],
    pub reserved_tail: [u8; CONFIG_POLICY_V2_TYPED_RESERVED_TAIL_BYTES],
}

impl Default for ConfigPolicyV2 {
    fn default() -> Self {
        Self {
            bump: 0,
            minimum_bundle_auction_pairs: 2,
            policy_flags: 0,
            _reserved0: [0; 8],
            admin_authorities: [Pubkey::default(); CONFIG_POLICY_V2_ADMIN_CAPACITY],
            service_authorities: [Pubkey::default(); CONFIG_POLICY_V2_SERVICE_CAPACITY],
            v2_verifiers_per_auction: VERIFIERS_PER_AUCTION as u8,
            v2_verifier_quorum: V2_VERIFIER_QUORUM as u8,
            _reserved1: [0; 6],
            tier_configs: [
                RequestTierConfigV2::from_request_tier(RequestTier::Eco),
                RequestTierConfigV2::from_request_tier(RequestTier::Small),
                RequestTierConfigV2::from_request_tier(RequestTier::Standard),
                RequestTierConfigV2::from_request_tier(RequestTier::Pro),
                RequestTierConfigV2::from_request_tier(RequestTier::Large),
            ],
            reserved_words: [[0; 32]; CONFIG_POLICY_V2_TYPED_RESERVED_WORDS],
            v2_account_layout_version: AccountLayoutVersion::V2 as u8,
            _reserved2: [0; CONFIG_POLICY_V2_TYPED_RESERVED_LAYOUT_PADDING_BYTES],
            reserved_tail: [0; CONFIG_POLICY_V2_TYPED_RESERVED_TAIL_BYTES],
        }
    }
}

impl ConfigPolicyV2 {
    pub const LEN: usize = std::mem::size_of::<ConfigPolicyV2>();

    pub fn tier_config(&self, tier: RequestTier) -> &RequestTierConfigV2 {
        &self.tier_configs[match tier {
            RequestTier::Eco => 0,
            RequestTier::Small => 1,
            RequestTier::Standard => 2,
            RequestTier::Pro => 3,
            RequestTier::Large => 4,
        }]
    }

    pub fn configured_v2_account_layout_version(&self) -> Result<AccountLayoutVersion, u8> {
        match AccountLayoutVersion::try_from(self.v2_account_layout_version) {
            Ok(version @ (AccountLayoutVersion::V1 | AccountLayoutVersion::V2)) => Ok(version),
            _ => Err(self.v2_account_layout_version),
        }
    }
}
