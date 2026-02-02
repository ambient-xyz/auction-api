use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum RequestTier {
    Eco = 0,
    Standard = 1,
    Pro = 2,
}
unsafe impl Pod for RequestTier {}

impl RequestTier {
    const ALL: [RequestTier; 3] = [RequestTier::Eco, RequestTier::Standard, RequestTier::Pro];

    pub fn get_verifiers_per_auction(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
        }
    }

    pub fn get_bid_reveal_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
        }
    }

    pub fn get_active_auction_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
        }
    }

    pub fn get_bundle_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 150,
            RequestTier::Standard => 150,
            RequestTier::Pro => 150,
        }
    }

    /// The maximum number of requests per bundle
    pub fn get_request_per_bundle(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 1,
            RequestTier::Pro => 1,
        }
    }

    /// Maximum allowed context length (in tokens) per tier
    pub fn get_max_context_length_tokens(&self) -> u64 {
        match self {
            RequestTier::Eco => 43_000,
            RequestTier::Standard => 86_000,
            RequestTier::Pro => 200_000,
        }
    }
    /// TODO: this should be enforced
    pub fn get_job_submission_duration_slots(&self) -> u64 {
        match self {
            RequestTier::Eco => 155,
            RequestTier::Standard => 145,
            RequestTier::Pro => 135,
        }
    }
    pub fn get_bid_commitment_amount_multiplier(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 2,
            RequestTier::Pro => 3,
        }
    }
    pub fn get_auction_credits_multiplier(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 2,
            RequestTier::Pro => 3,
        }
    }

    pub fn context_tier_for_tokens(tokens: u64) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|tier| tokens <= tier.get_max_context_length_tokens())
            .copied()
    }
}
