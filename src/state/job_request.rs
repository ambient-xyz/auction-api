use super::Pubkey;
use crate::state::request_tier::RequestTier;
use crate::state::verification::VerificationState;
use crate::{constant::PUBKEY_BYTES, MaybePubkey};
use bytemuck::{CheckedBitPattern, NoUninit, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Zeroable, NoUninit, CheckedBitPattern, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
/// Account created and owned by the auction program to keep state related to a user inference request.
pub struct JobRequest {
    /// The public key of the bundle this request is participating in
    pub bundle: Pubkey,
    /// The maximum price per output token
    pub max_price_per_output_token: u64,
    /// The maximum output token this request accepts
    pub max_output_tokens: u64,
    /// Context length tier type
    pub context_length_tier: RequestTier,
    /// Expiry duration tier type
    pub expiry_duration_tier: RequestTier,
    /// The public key of the job requester.
    pub authority: Pubkey,
    /// An [IPFS content identifier](https://docs.ipfs.tech/concepts/content-addressing/) of the metadata necessary to complete the job.
    pub input_hash: Pubkey,
    pub input_hash_iv: [u8; 16],
    /// seeds used to derive the request PDA
    pub seed: [u8; 32],
    /// bump for the request account
    pub bump: u64,
    /// Output tokens generated
    pub output_token_count: u64,
    /// Request input tokens
    pub input_token_count: u64,
    /// Current lifecycle stage of the job request.
    ///
    /// Indicates whether the request is still awaiting inference,
    /// pending verification, or already completed.
    pub status: JobRequestStatus,
    pub verification: VerificationState,
    pub input_data_account: MaybePubkey,
    pub output_data_account: MaybePubkey,
}

impl JobRequest {
    pub const LEN: usize = std::mem::size_of::<JobRequest>();
}

#[derive(Clone, Copy, Zeroable, NoUninit, CheckedBitPattern, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
/// Represents the lifecycle status of a job request.
pub enum JobRequestStatus {
    #[default]
    /// The request has been created and is waiting for inference output.
    WaitingForOutput = 0,
    /// The inference output has been generated and is awaiting verification.
    OutputReceived = 1,
    /// The output has been verified and the request is completed.
    OutputVerified = 2,
}

impl std::fmt::Display for JobRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            JobRequestStatus::WaitingForOutput => "WaitingForOutput",
            JobRequestStatus::OutputReceived => "OutputReceived",
            JobRequestStatus::OutputVerified => "OutputVerified",
        };
        write!(f, "{t}")
    }
}

impl Default for JobRequest {
    fn default() -> Self {
        Self {
            max_price_per_output_token: 0,
            max_output_tokens: 0,
            context_length_tier: RequestTier::Eco,
            expiry_duration_tier: RequestTier::Eco,
            bundle: Pubkey::default(),
            authority: Pubkey::default(),
            input_hash: Default::default(),
            seed: [0; PUBKEY_BYTES],
            bump: 0,
            output_token_count: 0,
            input_token_count: 0,
            status: Default::default(),
            verification: Default::default(),
            input_hash_iv: Default::default(),
            input_data_account: None.into(),
            output_data_account: None.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JobRequestStatus;

    #[test]
    fn job_request_status_rejects_invalid_discriminants() {
        let bytes = 99u64.to_le_bytes();
        assert!(bytemuck::checked::try_from_bytes::<JobRequestStatus>(&bytes).is_err());
    }

    #[test]
    fn job_request_status_has_stable_u64_layout() {
        let expected = 1u64.to_le_bytes();
        assert_eq!(
            std::mem::size_of::<JobRequestStatus>(),
            std::mem::size_of::<u64>()
        );
        assert_eq!(
            std::mem::align_of::<JobRequestStatus>(),
            std::mem::align_of::<u64>()
        );
        assert_eq!(
            bytemuck::bytes_of(&JobRequestStatus::OutputReceived),
            expected.as_slice()
        );
    }
}
