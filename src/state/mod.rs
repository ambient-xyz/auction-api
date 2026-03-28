pub mod auction;
pub mod bid;
pub mod bundle;
pub mod bundle_registry;
pub mod config;
pub use auction::*;
pub use bid::*;
pub mod job_request;
pub mod metadata;
pub mod request_tier;
mod verification;

pub use bundle::*;
pub use bundle_registry::*;
pub use config::*;
pub use job_request::*;
pub use metadata::*;
pub use request_tier::*;
pub use verification::*;

#[cfg(feature = "serde")]
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{CheckedBitPattern, NoUninit, Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::mem;
#[derive(Pod, PartialEq, Eq, Debug, Clone, Copy, Zeroable, Default)]
#[cfg_attr(
    feature = "serde",
    derive(BorshSerialize, BorshDeserialize, Deserialize)
)]
#[repr(transparent)]
pub struct Pubkey {
    key: [u8; 32],
}

#[cfg(feature = "serde")]
impl Serialize for Pubkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&bs58::encode(&self.key).into_string())
    }
}

impl Pubkey {
    pub fn inner(&self) -> [u8; 32] {
        self.key
    }
}

impl From<[u8; 32]> for Pubkey {
    fn from(key: [u8; 32]) -> Self {
        Self { key }
    }
}

impl PartialEq<[u8; 32]> for Pubkey {
    fn eq(&self, other: &[u8; 32]) -> bool {
        &self.key == other
    }
}

impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        self.key.as_slice()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct MaybePubkey(Pubkey);

impl MaybePubkey {
    pub fn get(&self) -> Option<Pubkey> {
        if self.0 == Pubkey::default() {
            None
        } else {
            Some(self.0)
        }
    }
}

impl From<Pubkey> for MaybePubkey {
    fn from(pk: Pubkey) -> Self {
        Self(pk)
    }
}

impl From<[u8; 32]> for MaybePubkey {
    fn from(pk: [u8; 32]) -> Self {
        Self(pk.into())
    }
}

impl From<Option<Pubkey>> for MaybePubkey {
    fn from(pk: Option<Pubkey>) -> Self {
        if let Some(pk) = pk {
            Self(pk)
        } else {
            Self(Pubkey::default())
        }
    }
}

impl From<MaybePubkey> for Option<Pubkey> {
    fn from(pk: MaybePubkey) -> Option<Pubkey> {
        if pk.0 == Pubkey::default() {
            None
        } else {
            Some(pk.0)
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for MaybePubkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&bs58::encode(&self.0).into_string())
    }
}

/// Shared account decoding helpers for both checked and legacy bytemuck-backed wire types.
pub trait AccountData: Sized + Copy + NoUninit + CheckedBitPattern {
    fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::checked::CheckedCastError> {
        bytemuck::checked::try_from_bytes(bytes)
    }

    fn try_from_bytes_mut(
        bytes: &mut [u8],
    ) -> Result<&mut Self, bytemuck::checked::CheckedCastError> {
        bytemuck::checked::try_from_bytes_mut(bytes)
    }

    fn try_read_unaligned(bytes: &[u8]) -> Result<Self, bytemuck::checked::CheckedCastError> {
        bytemuck::checked::try_pod_read_unaligned(bytes)
    }
}

impl<T> AccountData for T where T: Sized + Copy + NoUninit + CheckedBitPattern {}

/// Safely reads a `Pod` value from bytes at a given offset.
fn read_field<T: AccountData>(bytes: &[u8], offset: usize) -> Option<T> {
    let end = offset + mem::size_of::<T>();
    T::try_read_unaligned(bytes.get(offset..end)?).ok()
}
/// Safely writes a `Pod` value into a mutable byte slice at a given offset.
///
/// Returns `true` if the writing succeeds, or `false` if the slice is too small.
fn write_field<T: NoUninit>(bytes: &mut [u8], offset: usize, value: T) -> bool {
    let size = std::mem::size_of::<T>();
    if bytes.len() < offset + size {
        return false; // slice too small
    }

    let slice = &mut bytes[offset..offset + size];
    slice.copy_from_slice(bytemuck::bytes_of(&value));
    true
}
