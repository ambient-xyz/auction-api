//! The members of an exact-replay class: node keys of miners that run the class's engine and can replay its windows.
//!
//! Verifier selection for an exact-replay escrow draws only members, stake-weighted from the epoch's verifier
//! registry. A node joins and leaves with its own signature; an admin can remove a member.

use super::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, ParsedAccountLayout, Pubkey,
};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

pub const REPLAY_CLASS_V1_CAPACITY: usize = 64;

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RawReplayClassV1Data {
    /// SHA-256 of the class name; a seed of the account.
    pub class_id: [u8; 32],
    pub member_count: u16,
    pub bump: u8,
    pub _reserved0: [u8; 5],
    /// Strictly increasing node keys in `[..member_count]`; the rest are zero.
    #[cfg_attr(feature = "serde", serde(with = "serde_members"))]
    pub members: [Pubkey; REPLAY_CLASS_V1_CAPACITY],
}

pub type ReplayClassV1 = RawReplayClassV1Data;

impl Default for RawReplayClassV1Data {
    fn default() -> Self {
        Zeroable::zeroed()
    }
}

impl RawReplayClassV1Data {
    pub const PAYLOAD_LEN: usize = std::mem::size_of::<RawReplayClassV1Data>();
    pub const LEN: usize = AccountHeaderV1::LEN + Self::PAYLOAD_LEN;

    pub fn members(&self) -> &[Pubkey] {
        &self.members[..usize::from(self.member_count).min(REPLAY_CLASS_V1_CAPACITY)]
    }

    pub fn contains(&self, node: &Pubkey) -> bool {
        self.members()
            .binary_search_by(|member| member.inner().cmp(&node.inner()))
            .is_ok()
    }

    /// Whether the member list is canonical: strictly increasing, no zero key, zero past the count.
    pub fn is_canonical(&self) -> bool {
        let count = usize::from(self.member_count);
        count <= REPLAY_CLASS_V1_CAPACITY
            && self.members[..count]
                .windows(2)
                .all(|pair| pair[0].inner() < pair[1].inner())
            && self.members[..count]
                .iter()
                .all(|m| *m != Pubkey::default())
            && self.members[count..]
                .iter()
                .all(|m| *m == Pubkey::default())
            && self._reserved0 == [0; 5]
    }

    /// Insert in order. False if the node is already a member or the class is full.
    pub fn insert(&mut self, node: Pubkey) -> bool {
        let count = usize::from(self.member_count);
        if count >= REPLAY_CLASS_V1_CAPACITY || node == Pubkey::default() {
            return false;
        }
        match self.members[..count].binary_search_by(|member| member.inner().cmp(&node.inner())) {
            Ok(_) => false,
            Err(at) => {
                self.members.copy_within(at..count, at + 1);
                self.members[at] = node;
                self.member_count += 1;
                true
            }
        }
    }

    /// Remove. False if the node is not a member.
    pub fn remove(&mut self, node: &Pubkey) -> bool {
        let count = usize::from(self.member_count);
        match self.members[..count].binary_search_by(|member| member.inner().cmp(&node.inner())) {
            Ok(at) => {
                self.members.copy_within(at + 1..count, at);
                self.members[count - 1] = Pubkey::default();
                self.member_count -= 1;
                true
            }
            Err(_) => false,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<ReplayClassV1Ref<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }
        let (header_bytes, raw_bytes) = bytes.split_at(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::ReplayClassV1
            || layout.version != AccountLayoutVersion::V1
        {
            return None;
        }
        let raw = bytemuck::try_from_bytes::<RawReplayClassV1Data>(raw_bytes).ok()?;
        Some(ReplayClassV1Ref { header, raw })
    }

    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<ReplayClassV1Mut<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }
        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes_mut::<AccountHeaderV1>(header_bytes).ok()?;
        let layout = header.layout()?;
        if layout.discriminator != AccountDiscriminator::ReplayClassV1
            || layout.version != AccountLayoutVersion::V1
        {
            return None;
        }
        let raw = bytemuck::try_from_bytes_mut::<RawReplayClassV1Data>(raw_bytes).ok()?;
        Some(ReplayClassV1Mut { header, raw })
    }

    /// Write a fresh account: the V1 header, then this payload.
    pub fn write_bytes(&self, bytes: &mut [u8]) -> bool {
        if bytes.len() != Self::LEN {
            return false;
        }
        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        header_bytes.copy_from_slice(bytemuck::bytes_of(&AccountHeaderV1::new(
            AccountDiscriminator::ReplayClassV1,
        )));
        raw_bytes.copy_from_slice(bytemuck::bytes_of(self));
        true
    }
}

#[cfg(feature = "serde")]
mod serde_members {
    use super::{Pubkey, REPLAY_CLASS_V1_CAPACITY};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        members: &[Pubkey; REPLAY_CLASS_V1_CAPACITY],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        members.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<[Pubkey; REPLAY_CLASS_V1_CAPACITY], D::Error> {
        let members = Vec::<Pubkey>::deserialize(deserializer)?;
        members
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 members"))
    }
}

#[derive(Debug)]
pub struct ReplayClassV1Ref<'a> {
    header: &'a AccountHeaderV1,
    raw: &'a RawReplayClassV1Data,
}

#[derive(Debug)]
pub struct ReplayClassV1Mut<'a> {
    header: &'a mut AccountHeaderV1,
    raw: &'a mut RawReplayClassV1Data,
}

impl ReplayClassV1Ref<'_> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }
}

impl Deref for ReplayClassV1Ref<'_> {
    type Target = RawReplayClassV1Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl ReplayClassV1Mut<'_> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }
}

impl Deref for ReplayClassV1Mut<'_> {
    type Target = RawReplayClassV1Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl DerefMut for ReplayClassV1Mut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw
    }
}
