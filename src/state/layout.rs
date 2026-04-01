//! Shared account layout identifiers for versioned account-state rollouts.
//!
//! When additional account types adopt this model, keep the shape simple:
//! - raw payload struct: `RawXData`
//! - byte-backed accessors: `XRef` / `XMut`
//! - layout classifier: `parse_x_layout`
//! - legacy size constant: `LEGACY_LEN`

use bytemuck::Zeroable;
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
