use bytemuck::{CheckedBitPattern, NoUninit, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    TryFromPrimitive,
    IntoPrimitive,
    Zeroable,
    NoUninit,
    CheckedBitPattern,
    Hash,
)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum RequestTier {
    Eco = 0,
    Small = 3,
    Standard = 1,
    Pro = 2,
    Large = 4,
}

impl RequestTier {
    pub const ALL: [RequestTier; 5] = [
        RequestTier::Eco,
        RequestTier::Small,
        RequestTier::Standard,
        RequestTier::Pro,
        RequestTier::Large,
    ];

    pub fn get_verifiers_per_auction(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
            RequestTier::Small => 3,
        }
    }

    pub fn get_bid_reveal_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
            RequestTier::Small => 3,
        }
    }

    pub fn get_active_auction_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 3,
            RequestTier::Standard => 3,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
            RequestTier::Small => 3,
        }
    }

    pub fn get_bundle_duration(&self) -> u64 {
        match self {
            RequestTier::Eco => 25,
            RequestTier::Standard => 25,
            RequestTier::Pro => 25,
            RequestTier::Large => 25,
            RequestTier::Small => 25,
        }
    }

    /// The maximum number of requests per bundle
    pub fn get_request_per_bundle(&self) -> u64 {
        match self {
            RequestTier::Eco => 30,
            RequestTier::Small => 12,
            RequestTier::Standard => 5,
            RequestTier::Pro => 2,
            RequestTier::Large => 1,
        }
    }

    /// Maximum allowed context length (in tokens) per tier
    pub fn get_max_context_length_tokens(&self) -> u64 {
        match self {
            RequestTier::Eco => 2_000,
            RequestTier::Small => 16_000,
            RequestTier::Standard => 32_000,
            RequestTier::Pro => 64_000,
            RequestTier::Large => 202_752,
        }
    }
    /// TODO: this should be enforced
    pub fn get_job_submission_duration_slots(&self) -> u64 {
        match self {
            RequestTier::Eco => 155,
            RequestTier::Standard => 145,
            RequestTier::Small => 145,
            RequestTier::Pro => 135,
            RequestTier::Large => 125,
        }
    }
    pub fn get_bid_commitment_amount_multiplier(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 2,
            RequestTier::Small => 2,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
        }
    }
    pub fn get_auction_credits_multiplier(&self) -> u64 {
        match self {
            RequestTier::Eco => 1,
            RequestTier::Standard => 2,
            RequestTier::Small => 2,
            RequestTier::Pro => 3,
            RequestTier::Large => 3,
        }
    }

    pub fn context_tier_for_tokens(tokens: u64) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|tier| tokens <= tier.get_max_context_length_tokens())
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::RequestTier;

    #[test]
    fn request_tier_rejects_invalid_discriminants() {
        let bytes = 99u64.to_le_bytes();
        assert!(bytemuck::checked::try_from_bytes::<RequestTier>(&bytes).is_err());
    }

    #[test]
    fn request_tier_has_stable_u64_layout() {
        let expected = 2u64.to_le_bytes();
        assert_eq!(std::mem::size_of::<RequestTier>(), std::mem::size_of::<u64>());
        assert_eq!(std::mem::align_of::<RequestTier>(), std::mem::align_of::<u64>());
        assert_eq!(bytemuck::bytes_of(&RequestTier::Pro), expected.as_slice());
    }
}
