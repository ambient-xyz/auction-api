use super::{Pubkey, RequestTier};
use crate::VERIFIERS_PER_AUCTION;
use bytemuck::{Pod, Zeroable};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(into = "u64", try_from = "u64"))]
#[repr(transparent)]
pub struct BundleEscrowV2Status(u64);

#[allow(non_upper_case_globals)]
impl BundleEscrowV2Status {
    pub const Open: Self = Self(0);
    pub const Awarded: Self = Self(1);
    pub const ResultPosted: Self = Self(2);
    pub const FinalizedVerified: Self = Self(3);
    pub const FinalizedRejected: Self = Self(4);
    pub const Expired: Self = Self(5);

    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

impl Default for BundleEscrowV2Status {
    fn default() -> Self {
        Self::Open
    }
}

impl From<BundleEscrowV2Status> for u64 {
    fn from(value: BundleEscrowV2Status) -> Self {
        value.0
    }
}

impl TryFrom<u64> for BundleEscrowV2Status {
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::Awarded),
            2 => Ok(Self::ResultPosted),
            3 => Ok(Self::FinalizedVerified),
            4 => Ok(Self::FinalizedRejected),
            5 => Ok(Self::Expired),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BundleEscrowV2Status;

    #[test]
    fn bundle_escrow_v2_status_round_trips_through_raw_values() {
        assert_eq!(BundleEscrowV2Status::try_from(0), Ok(BundleEscrowV2Status::Open));
        assert_eq!(BundleEscrowV2Status::try_from(5), Ok(BundleEscrowV2Status::Expired));
        assert_eq!(u64::from(BundleEscrowV2Status::Awarded), 1);
        assert_eq!(BundleEscrowV2Status::try_from(99), Err(99));
    }

    #[test]
    fn bundle_escrow_v2_status_matches_on_associated_constants() {
        let label = match BundleEscrowV2Status::ResultPosted {
            BundleEscrowV2Status::Open => "open",
            BundleEscrowV2Status::Awarded => "awarded",
            BundleEscrowV2Status::ResultPosted => "result-posted",
            BundleEscrowV2Status::FinalizedVerified => "finalized-verified",
            BundleEscrowV2Status::FinalizedRejected => "finalized-rejected",
            BundleEscrowV2Status::Expired => "expired",
            _ => "invalid",
        };

        assert_eq!(label, "result-posted");
    }
}
