use super::{Pubkey, RequestTier};
use crate::VERIFIERS_PER_AUCTION;
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct BundleEscrowV2 {
    pub status: BundleEscrowV2Status,
    pub reward_tier: RequestTier,
    pub coordinator: Pubkey,
    pub requester_refund_recipient: Pubkey,
    pub bundle_version: u32,
    pub _reserved0: [u8; 4],
    pub bundle_hash: [u8; 32],
    pub total_input_tokens: u64,
    pub max_output_tokens: u64,
    pub winner_node_pubkey: Pubkey,
    pub winner_vote_account: Pubkey,
    pub clearing_price_per_output_token: u64,
    pub selected_verifiers: [Pubkey; VERIFIERS_PER_AUCTION],
    pub auction_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub verification_hash: [u8; 32],
    pub posted_output_tokens: u64,
    pub accepted_output_tokens: u64,
    pub settlement_deadline_slot: u64,
    pub result_deadline_slot: u64,
    pub verification_deadline_slot: u64,
    pub claim_deadline_slot: u64,
    pub winner_reward_claimed: u8,
    pub verifier_reward_claimed_bitmap: u8,
    pub quorum_verifier_bitmap: u8,
    pub _reserved1: [u8; 5],
}

impl BundleEscrowV2 {
    pub const LEN: usize = std::mem::size_of::<BundleEscrowV2>();

    pub fn all_quorum_verifier_rewards_claimed(&self) -> bool {
        self.verifier_reward_claimed_bitmap & self.quorum_verifier_bitmap
            == self.quorum_verifier_bitmap
    }

    pub fn final_reward_claims_complete(&self) -> bool {
        let winner_done = self.status != BundleEscrowV2Status::FinalizedVerified
            || self.winner_reward_claimed != 0;
        winner_done && self.all_quorum_verifier_rewards_claimed()
    }
}

impl Default for BundleEscrowV2 {
    fn default() -> Self {
        Self {
            status: BundleEscrowV2Status::Open,
            reward_tier: RequestTier::Eco,
            coordinator: Pubkey::default(),
            requester_refund_recipient: Pubkey::default(),
            bundle_version: 0,
            _reserved0: [0; 4],
            bundle_hash: [0; 32],
            total_input_tokens: 0,
            max_output_tokens: 0,
            winner_node_pubkey: Pubkey::default(),
            winner_vote_account: Pubkey::default(),
            clearing_price_per_output_token: 0,
            selected_verifiers: [Pubkey::default(); VERIFIERS_PER_AUCTION],
            auction_hash: [0; 32],
            result_hash: [0; 32],
            verification_hash: [0; 32],
            posted_output_tokens: 0,
            accepted_output_tokens: 0,
            settlement_deadline_slot: 0,
            result_deadline_slot: 0,
            verification_deadline_slot: 0,
            claim_deadline_slot: 0,
            winner_reward_claimed: 0,
            verifier_reward_claimed_bitmap: 0,
            quorum_verifier_bitmap: 0,
            _reserved1: [0; 5],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum BundleEscrowV2Status {
    Open = 0,
    Awarded = 1,
    ResultPosted = 2,
    FinalizedVerified = 3,
    FinalizedRejected = 4,
    Expired = 5,
}

unsafe impl Pod for BundleEscrowV2Status {}
