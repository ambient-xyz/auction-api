pub mod auction;
pub mod bid;
pub mod bundle;
pub mod bundle_escrow_v2;
pub mod bundle_registry;
pub mod bundle_verifier_page_v2;
pub mod config;
pub mod layout;
pub use auction::*;
pub use bid::*;
pub mod job_request;
pub mod metadata;
pub mod request_tier;
mod verification;

pub use bundle::*;
pub use bundle_escrow_v2::*;
pub use bundle_registry::*;
pub use bundle_verifier_page_v2::*;
pub use config::*;
pub use job_request::*;
pub use layout::*;
pub use metadata::*;
pub use request_tier::*;
pub use verification::*;

#[cfg(feature = "serde")]
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
