//! Shared account layout identifiers for versioned account-state rollouts.
//!
//! When additional account types adopt this model, keep the shape simple:
//! - raw payload struct: `RawXData`
//! - byte-backed accessors: `XRef` / `XMut`
//! - layout classifier: `parse_x_layout`
//! - legacy size constant: `LEGACY_LEN`

use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[repr(u8)]
pub enum AccountDiscriminator {
    Unknown = 0,
    Bundle = 1,
    Auction = 2,
    Bid = 3,
    JobRequest = 4,
    BundleRegistry = 5,
    Config = 6,
    Metadata = 7,
    BundleEscrowV2 = 8,
    BundleVerifierPageV2 = 9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Zeroable)]
#[repr(u8)]
pub enum AccountLayoutVersion {
    LegacyV0 = 0,
    V1 = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParsedAccountLayout {
    pub discriminator: AccountDiscriminator,
    pub version: AccountLayoutVersion,
}

impl ParsedAccountLayout {
    pub const fn new(discriminator: AccountDiscriminator, version: AccountLayoutVersion) -> Self {
        Self {
            discriminator,
            version,
        }
    }

    pub const fn legacy_v0(discriminator: AccountDiscriminator) -> Self {
        Self::new(discriminator, AccountLayoutVersion::LegacyV0)
    }

    pub const fn is_legacy(self) -> bool {
        matches!(self.version, AccountLayoutVersion::LegacyV0)
    }
}

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct AccountHeaderV1 {
    pub discriminator: u8,
    pub version: u8,
    pub reserved: [u8; 6],
}

impl AccountHeaderV1 {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub const fn new(discriminator: AccountDiscriminator) -> Self {
        Self {
            discriminator: discriminator as u8,
            version: AccountLayoutVersion::V1 as u8,
            reserved: [0; 6],
        }
    }

    pub fn layout(&self) -> Option<ParsedAccountLayout> {
        let discriminator = AccountDiscriminator::try_from(self.discriminator).ok()?;
        let version = AccountLayoutVersion::try_from(self.version).ok()?;
        Some(ParsedAccountLayout::new(discriminator, version))
    }
}
