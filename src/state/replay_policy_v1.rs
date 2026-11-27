//! Exact-replay policy terms, stored in `ConfigPolicyV2.replay` (the three former reserved words at offset 1448).

use super::RequestTier;
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The policy is off at version 0 (every byte zero) and on at version 1.
pub const REPLAY_POLICY_V1_VERSION: u8 = 1;
/// Penalty slots in [`ReplayPolicyV1::suspension_hours`]; kinds past [`ReplayFaultKind::COUNT`] are reserved.
pub const REPLAY_FAULT_KIND_CAPACITY: usize = 15;
/// Upper bound on the off-chain arbitration round size.
pub const MAX_REPLAY_ARBITERS: u8 = 7;

/// A fault the exact-replay coordinator can record against a miner or a verifier. The index selects the penalty terms
/// in [`ReplayPolicyV1`]; the names match the coordinator's fault kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplayFaultKind {
    Mismatch = 0,
    IoMismatch = 1,
    InvalidOpening = 2,
    ClassMismatch = 3,
    UnsupportedPreemption = 4,
    MissingOpening = 5,
    DigestDisagrees = 6,
    CanaryPassed = 7,
    FalseFlag = 8,
    OutcomeDisagrees = 9,
    BadReveal = 10,
}

impl ReplayFaultKind {
    pub const COUNT: usize = 11;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Mismatch,
        Self::IoMismatch,
        Self::InvalidOpening,
        Self::ClassMismatch,
        Self::UnsupportedPreemption,
        Self::MissingOpening,
        Self::DigestDisagrees,
        Self::CanaryPassed,
        Self::FalseFlag,
        Self::OutcomeDisagrees,
        Self::BadReveal,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mismatch => "mismatch",
            Self::IoMismatch => "io_mismatch",
            Self::InvalidOpening => "invalid_opening",
            Self::ClassMismatch => "class_mismatch",
            Self::UnsupportedPreemption => "unsupported_preemption",
            Self::MissingOpening => "missing_opening",
            Self::DigestDisagrees => "digest_disagrees",
            Self::CanaryPassed => "canary_passed",
            Self::FalseFlag => "false_flag",
            Self::OutcomeDisagrees => "outcome_disagrees",
            Self::BadReveal => "bad_reveal",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == name)
    }
}

/// Exact-replay terms. 96 bytes: word 0 is the class, word 1 the lane and hold, word 2 the penalties.
#[derive(Pod, Clone, Copy, Zeroable, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct ReplayPolicyV1 {
    /// SHA-256 of the replay class name (model, engine, GPU and TP size) whose members verify replay-lane escrows.
    pub class_id: [u8; 32],
    /// 0: exact replay is off and every byte is zero. 1: on.
    pub version: u8,
    /// Bit `1 << tier` for each [`RequestTier`] that is an exact-replay lane. Small is never one.
    pub tier_mask: u8,
    /// Fresh verifiers in an off-chain arbitration round.
    pub arbiters: u8,
    pub _reserved0: [u8; 5],
    /// The vesting window: a replay-lane escrow finalizes provisionally and pays out only once it has passed. It is
    /// also the window in which its verdict can be disputed.
    pub hold_slots: u64,
    pub _reserved1: [u8; 16],
    /// Bit `1 << kind` for each [`ReplayFaultKind`] whose fault forfeits the party's unvested winnings.
    pub forfeit_mask: u16,
    /// Suspension per [`ReplayFaultKind`], in hours.
    pub suspension_hours: [u16; REPLAY_FAULT_KIND_CAPACITY],
}

impl ReplayPolicyV1 {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub fn enabled(&self) -> bool {
        self.version == REPLAY_POLICY_V1_VERSION
    }

    /// Whether escrows of this tier are exact-replay escrows under this policy.
    pub fn is_replay_tier(&self, tier: RequestTier) -> bool {
        self.enabled() && self.tier_mask & tier_bit(tier) != 0
    }

    pub fn forfeits(&self, kind: ReplayFaultKind) -> bool {
        self.forfeit_mask & (1 << kind as u16) != 0
    }

    pub fn suspension_hours(&self, kind: ReplayFaultKind) -> u16 {
        self.suspension_hours[kind as usize]
    }

    /// Structural rules. Rules that relate the hold to other policy terms are checked with the whole policy, by the
    /// program.
    pub fn validate(&self) -> bool {
        if self.version == 0 {
            return *self == Self::default();
        }
        let allowed_tiers = RequestTier::ALL
            .into_iter()
            .filter(|tier| *tier != RequestTier::Small)
            .fold(0u8, |mask, tier| mask | tier_bit(tier));
        let known_kinds = (1u32 << ReplayFaultKind::COUNT) - 1;
        self.version == REPLAY_POLICY_V1_VERSION
            && self.class_id != [0; 32]
            && self.tier_mask != 0
            && self.tier_mask & !allowed_tiers == 0
            && (1..=MAX_REPLAY_ARBITERS).contains(&self.arbiters)
            && self._reserved0 == [0; 5]
            && self.hold_slots != 0
            && self.hold_slots <= u64::from(u32::MAX)
            && self._reserved1 == [0; 16]
            && u32::from(self.forfeit_mask) & !known_kinds == 0
            && self.suspension_hours[ReplayFaultKind::COUNT..]
                .iter()
                .all(|hours| *hours == 0)
    }
}

/// The tier's bit in [`ReplayPolicyV1::tier_mask`].
pub fn tier_bit(tier: RequestTier) -> u8 {
    1u8 << u64::from(tier)
}
