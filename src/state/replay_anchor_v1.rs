//! A miner's on-chain anchor of one engine's span commits (exact replay).
//!
//! Span commits chain: each names the SHA-256 of the previous span's commit. Anchoring the SHA-256 of span N's commit
//! therefore fixes every commit of spans up to N. The coordinator draws challenges from a slot after the anchor, so a
//! miner cannot choose its history after seeing which spans are challenged.

use super::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, ParsedAccountLayout, Pubkey,
};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Anchors kept in the account, newest last in ring order. Older ones remain in transaction history.
pub const REPLAY_ANCHOR_V1_RECENT: usize = 8;

#[derive(Pod, Clone, Copy, Zeroable, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct ReplayAnchorEntryV1 {
    pub span_id: u64,
    pub slot: u64,
    pub commit_sha256: [u8; 32],
}

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RawReplayAnchorV1Data {
    /// The miner's node key, which signs its span commits and every anchor.
    pub miner_node: Pubkey,
    /// SHA-256 of the engine id, as in the signed span commit.
    pub engine: [u8; 32],
    /// The span of the first anchor.
    pub first_span_id: u64,
    pub anchor_count: u64,
    pub bump: u8,
    pub _reserved0: [u8; 7],
    /// The last [`REPLAY_ANCHOR_V1_RECENT`] anchors; anchor `n` (0-based) is at `n % REPLAY_ANCHOR_V1_RECENT`.
    pub recent: [ReplayAnchorEntryV1; REPLAY_ANCHOR_V1_RECENT],
}

pub type ReplayAnchorV1 = RawReplayAnchorV1Data;

impl RawReplayAnchorV1Data {
    pub const PAYLOAD_LEN: usize = std::mem::size_of::<RawReplayAnchorV1Data>();
    pub const LEN: usize = AccountHeaderV1::LEN + Self::PAYLOAD_LEN;

    pub const fn account_len(version: AccountLayoutVersion) -> usize {
        match version {
            AccountLayoutVersion::V1 => Self::LEN,
            _ => 0,
        }
    }

    /// The newest anchor, or None before the first.
    pub fn latest(&self) -> Option<&ReplayAnchorEntryV1> {
        let n = self.anchor_count.checked_sub(1)?;
        Some(&self.recent[(n % REPLAY_ANCHOR_V1_RECENT as u64) as usize])
    }

    /// Record a new anchor. The caller has checked that `span_id` is past the latest.
    pub fn push(&mut self, entry: ReplayAnchorEntryV1) {
        if self.anchor_count == 0 {
            self.first_span_id = entry.span_id;
        }
        self.recent[(self.anchor_count % REPLAY_ANCHOR_V1_RECENT as u64) as usize] = entry;
        self.anchor_count += 1;
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<ReplayAnchorV1Ref<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }
        let (header_bytes, raw_bytes) = bytes.split_at(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::ReplayAnchorV1
            || layout.version != AccountLayoutVersion::V1
        {
            return None;
        }
        let raw = bytemuck::try_from_bytes::<RawReplayAnchorV1Data>(raw_bytes).ok()?;
        Some(ReplayAnchorV1Ref { header, raw })
    }

    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<ReplayAnchorV1Mut<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }
        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes_mut::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::ReplayAnchorV1
            || layout.version != AccountLayoutVersion::V1
        {
            return None;
        }
        let raw = bytemuck::try_from_bytes_mut::<RawReplayAnchorV1Data>(raw_bytes).ok()?;
        Some(ReplayAnchorV1Mut { header, raw })
    }

    /// Write a fresh account: the V1 header, then this payload.
    pub fn write_bytes(&self, bytes: &mut [u8]) -> bool {
        if bytes.len() != Self::LEN {
            return false;
        }
        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        header_bytes.copy_from_slice(bytemuck::bytes_of(&AccountHeaderV1::new(
            AccountDiscriminator::ReplayAnchorV1,
        )));
        raw_bytes.copy_from_slice(bytemuck::bytes_of(self));
        true
    }
}

impl Default for RawReplayAnchorV1Data {
    fn default() -> Self {
        Zeroable::zeroed()
    }
}

#[derive(Debug)]
pub struct ReplayAnchorV1Ref<'a> {
    header: &'a AccountHeaderV1,
    raw: &'a RawReplayAnchorV1Data,
}

#[derive(Debug)]
pub struct ReplayAnchorV1Mut<'a> {
    header: &'a mut AccountHeaderV1,
    raw: &'a mut RawReplayAnchorV1Data,
}

impl ReplayAnchorV1Ref<'_> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }
}

impl Deref for ReplayAnchorV1Ref<'_> {
    type Target = RawReplayAnchorV1Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl ReplayAnchorV1Mut<'_> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }
}

impl Deref for ReplayAnchorV1Mut<'_> {
    type Target = RawReplayAnchorV1Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl DerefMut for ReplayAnchorV1Mut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw
    }
}
