use super::Pubkey;
use crate::state::request_tier::RequestTier;
use crate::{constant::PUBKEY_BYTES, MaybePubkey, VERIFIERS_PER_AUCTION};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Pod)]
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

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
/// Holds all data required to manage and track verification of a job request.
///
/// Includes:
/// - Merkle root of the job’s output data.
/// - Assigned verifiers and their corresponding token ranges.
/// - Individual verifier states and verified token counts.
/// - Output hash for integrity checks (optionally encrypted).
/// - Initialization vectors (IVs) for optional encryption of the output hash
///   and Merkle root, using a shared secret between client and ambient.
pub struct VerificationState {
    pub merkle_root: [u8; 32],
    pub assigned_verifiers: [Pubkey; VERIFIERS_PER_AUCTION],
    pub assigned_verifiers_token_ranges: [u64; VERIFIERS_PER_AUCTION * 2],
    pub verifier_states: [JobVerificationState; VERIFIERS_PER_AUCTION],
    pub verified_tokens: [u64; VERIFIERS_PER_AUCTION],
    pub output_hash: [u8; 32],
    /// output hash and merkle root may be encrypted with a shared secret + iv,
    /// where shared_secret = ambient private key x client public key
    /// and IV is a random byte array (a nonce in crypto terms)
    ///
    /// encryption is used iff `encryption_iv` != [0; 16]
    pub output_hash_iv: [u8; 16],
    pub merkle_root_iv: [u8; 16],
}

impl JobRequest {
    pub const LEN: usize = std::mem::size_of::<JobRequest>();
}

#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Default)]
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

unsafe impl Pod for JobRequestStatus {}

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

#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum JobVerificationState {
    #[default]
    NotStarted = 0,
    InProgress = 1,
    Completed = 2,
}

unsafe impl Pod for JobVerificationState {}

impl std::fmt::Display for JobVerificationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            JobVerificationState::NotStarted => "NotStarted",
            JobVerificationState::InProgress => "InProgress",
            JobVerificationState::Completed => "Completed",
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
