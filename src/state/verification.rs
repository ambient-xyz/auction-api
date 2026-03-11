use crate::{error, Pubkey, VERIFIERS_PER_AUCTION};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

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
    pub verifier_states: [JobVerificationStateRaw; VERIFIERS_PER_AUCTION],
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

#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Default, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u64)]
pub enum JobVerificationState {
    #[default]
    NotStarted = 0,
    InProgress = 1,
    Completed = 2,
}

#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Default, Pod)]
#[repr(transparent)]
pub struct JobVerificationStateRaw(u64);

impl JobVerificationStateRaw {
    pub fn new(value: JobVerificationState) -> Self {
        value.into()
    }
}
impl From<JobVerificationState> for JobVerificationStateRaw {
    fn from(value: JobVerificationState) -> Self {
        Self(value.into())
    }
}

impl TryFrom<JobVerificationStateRaw> for JobVerificationState {
    type Error = error::AuctionError;

    fn try_from(value: JobVerificationStateRaw) -> Result<Self, Self::Error> {
        Self::try_from(value.0).map_err(|_| error::AuctionError::InvalidJobVerificationState)
    }
}

#[cfg(feature = "serde")]
impl Serialize for JobVerificationStateRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        JobVerificationState::try_from(*self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for JobVerificationStateRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        JobVerificationState::deserialize(deserializer)
            .map(JobVerificationStateRaw::from)
            .map_err(de::Error::custom)
    }
}

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

#[cfg(test)]
mod tests {
    use super::{JobVerificationState, JobVerificationStateRaw};
    use crate::error::AuctionError;
    use bytemuck::cast;

    #[test]
    fn verification_state_round_trips_through_raw_wrapper() {
        for state in [
            JobVerificationState::NotStarted,
            JobVerificationState::InProgress,
            JobVerificationState::Completed,
        ] {
            let raw = JobVerificationStateRaw::from(state);
            assert_eq!(JobVerificationState::try_from(raw), Ok(state));
        }
    }

    #[test]
    fn verification_state_rejects_invalid_raw_values() {
        let raw = cast::<u64, JobVerificationStateRaw>(99);
        assert_eq!(
            JobVerificationState::try_from(raw),
            Err(AuctionError::InvalidJobVerificationState)
        );
    }

    #[test]
    fn verification_state_raw_matches_u64_layout() {
        assert_eq!(
            std::mem::size_of::<JobVerificationStateRaw>(),
            std::mem::size_of::<u64>()
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn verification_state_raw_serde_matches_enum_representation() {
        let raw = JobVerificationStateRaw::from(JobVerificationState::Completed);
        let serialized = serde_json::to_string(&raw).unwrap();
        assert_eq!(serialized, "\"Completed\"");

        let deserialized: JobVerificationStateRaw = serde_json::from_str("\"InProgress\"").unwrap();
        assert_eq!(
            JobVerificationState::try_from(deserialized),
            Ok(JobVerificationState::InProgress)
        );
    }
}
