use super::{read_field, write_field, Pubkey};
use crate::constant::PUBKEY_BYTES;
use crate::state::request_tier::RequestTier;
use crate::{MaybePubkey, BUNDLE_DURATION, REQUESTS_PER_BUNDLE, VERIFIERS_PER_AUCTION};
use bytemuck::{offset_of, Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(Default, Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct Verifiers {
    pub keys: [Pubkey; VERIFIERS_PER_AUCTION],
}

/// A bundle is a block of economically similar requests
#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RequestBundle {
    /// Current status of the bundle
    pub status: BundleStatus,
    /// Context length tier type
    pub context_length_tier: RequestTier,
    /// Expiry duration tier type
    pub expiry_duration_tier: RequestTier,
    /// The auction for this bundle.
    pub auction: MaybePubkey,
    /// Assigned verifiers for this bundle.
    pub verifiers: Verifiers,
    /// The slot after which the auction cannot receive any more bids and is considered ended.
    pub expiry_slot: u64,
    /// The maximum input tokens each request can have
    pub max_context_length: u64,
    /// Total number of requests contained in this bundle.
    pub requests_len: u64,
    /// The number of job requests that were successfully verified
    pub num_verified_requests: u64,
    /// limit how much time winning bidder can take to submit all jobs
    pub job_submission_duration: u64,
    /// Total amount commited by the requesters
    pub request_committed_amount: u64,
    /// Total input tokens in the requests
    pub total_input_tokens: u64,
    /// Maximum output tokens to be generated for the requests
    pub maximum_output_tokens: u64,
    /// Total output tokens generated for the requests
    pub output_tokens_generated: u64,
    /// the parent bundle key is bundle is derived from
    pub parent_bundle_key: Pubkey,
    /// The child bundle key to be derived from this bundle
    pub child_bundle_key: MaybePubkey,
    /// bump for this bundle account
    pub bump: u64,
    /// assuming child_bundle bump is not zero (possible but statistically improbable)
    pub child_bundle_bump: Option<NonZeroU64>,
    /// assuming auction bump is not zero (possible but statistically improbable)
    pub auction_bump: Option<NonZeroU64>,
    /// payer key for the bundle account creation
    pub payer: Pubkey,
    /// The clearing price from the concluded auction for this bundle.
    /// Denotes the payment rate (in lamports) per output token that the
    /// winning bidder will receive for fulfilling the bundle’s requests.
    pub price_per_output_token: Option<NonZeroU64>,
}

impl RequestBundle {
    pub const LEN: usize = std::mem::size_of::<RequestBundle>();
    pub fn new(
        payer: [u8; PUBKEY_BYTES],
        parent_bundle_key: [u8; PUBKEY_BYTES],
        bump: u64,
        current_slot: u64,
        context_length_tier: RequestTier,
        expiry_duration_tier: RequestTier,
        max_context_length: u64,
    ) -> Self {
        RequestBundle {
            payer: payer.into(),
            parent_bundle_key: parent_bundle_key.into(),
            bump,
            expiry_slot: current_slot.saturating_add(BUNDLE_DURATION),
            context_length_tier,
            expiry_duration_tier,
            max_context_length,
            ..Default::default()
        }
    }
    pub fn add_request_record(
        &mut self,
        commited_amount: u64,
        input_tokens: u64,
        max_output_tokens: u64,
    ) {
        self.requests_len = self.requests_len.saturating_add(1);
        self.request_committed_amount = self
            .request_committed_amount
            .saturating_add(commited_amount);
        self.total_input_tokens = self.total_input_tokens.saturating_add(input_tokens);
        self.maximum_output_tokens = self.maximum_output_tokens.saturating_add(max_output_tokens);
    }

    fn read_expiry_slot_from_bytes(bytes: &[u8]) -> Option<u64> {
        let offset = offset_of!(RequestBundle, expiry_slot);
        read_field(bytes, offset)
    }
    fn read_requests_len_from_bytes(bytes: &[u8]) -> Option<u64> {
        let offset = offset_of!(RequestBundle, requests_len);
        read_field(bytes, offset)
    }
    pub fn cancel_bundle_from_bytes(bytes: &mut [u8]) -> bool {
        let offset = offset_of!(RequestBundle, status);
        write_field(bytes, offset, BundleStatus::Canceled)
    }
    pub fn is_expired_from_bytes(bytes: &[u8], slot: u64) -> Option<bool> {
        let requests_len = Self::read_requests_len_from_bytes(bytes)?;
        let expiry_slot = Self::read_expiry_slot_from_bytes(bytes)?;
        Some(requests_len < 1 && expiry_slot <= slot)
    }
}
impl Default for RequestBundle {
    fn default() -> Self {
        Self {
            requests_len: 0,
            job_submission_duration: RequestTier::Eco.get_job_submission_duration_slots(),
            request_committed_amount: 0,
            total_input_tokens: 0,
            maximum_output_tokens: 0,
            output_tokens_generated: 0,
            parent_bundle_key: Default::default(),
            child_bundle_key: Default::default(),
            num_verified_requests: 0,
            context_length_tier: RequestTier::Eco,
            expiry_duration_tier: RequestTier::Eco,
            auction: Default::default(),
            verifiers: Default::default(),
            expiry_slot: 0,
            max_context_length: RequestTier::Eco.get_max_context_length_tokens(),
            status: BundleStatus::Active,
            bump: 0,
            child_bundle_bump: None,
            auction_bump: None,
            payer: Default::default(),
            price_per_output_token: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum BundleStatus {
    /// The bundle is currently active and accepting requests.
    Active = 0,
    /// The bundle is filled, awaiting job output submission
    Full = 2,
    /// The auction job output is not validated yet
    PendingVerification = 3,
    /// The auction job output has been validated
    Verified = 4,
    /// The job output is invalid
    BadJobOutput = 5,
    /// The bundle has failed to conclude
    Canceled = 6,
}
unsafe impl Pod for BundleStatus {}
#[cfg(test)]
mod tests {
    use super::*;
    use memoffset::offset_of;

    // Verify that the memory layout of RequestBundle matches assumptions made
    // elsewhere in the system (e.g., raw writes or FFI). Changes that break
    // these tests may require updating dependent code.
    #[test]
    fn layout_offsets() {
        assert_eq!(offset_of!(RequestBundle, status), 0);
        assert_eq!(offset_of!(RequestBundle, context_length_tier), 8);
        assert_eq!(offset_of!(RequestBundle, expiry_duration_tier), 16);
        assert_eq!(offset_of!(RequestBundle, expiry_slot), 152);
        assert_eq!(offset_of!(RequestBundle, requests_len), 168);
    }

    #[test]
    fn layout_sizes() {
        assert_eq!(size_of::<RequestTier>(), 8);
        assert_eq!(size_of::<BundleStatus>(), 8);

        let request = RequestBundle::default();
        let _: BundleStatus = request.status;
        let _: RequestTier = request.context_length_tier;
        let _: RequestTier = request.expiry_duration_tier;
        let _: u64 = request.expiry_slot;
        let _: u64 = request.requests_len;
    }
}
