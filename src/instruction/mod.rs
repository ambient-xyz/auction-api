use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::net;

mod append_data;
mod cancel_bundle;
mod claim_verifier_lstake_v2;
mod claim_winner_lstake_v2;
mod close_bid;
mod close_request;
mod commit_auction_settlement_v2;
mod dispute_bundle_verification_v2;
mod end_auction;
mod expire_bundle_escrow_v2;
mod finalize_bundle_verification_v2;
mod init_bundle;
mod init_bundle_verifier_page_v2;
#[cfg(feature = "global-config")]
mod init_config;
mod init_config_policy_v2;
mod open_bundle_escrow_v2;
mod place_bid;
mod post_bundle_result_v2;
mod request_job;
mod reveal_bid;
mod select_bundle_verifiers_v2;
mod set_config_policy_v2;
mod submit_job_output;
mod submit_validation;

use crate::macros::impl_instruction_data;
pub use append_data::*;
pub use cancel_bundle::*;
pub use claim_verifier_lstake_v2::*;
pub use claim_winner_lstake_v2::*;
pub use close_bid::*;
pub use close_request::*;
pub use commit_auction_settlement_v2::*;
pub use dispute_bundle_verification_v2::*;
pub use end_auction::*;
pub use expire_bundle_escrow_v2::*;
pub use finalize_bundle_verification_v2::*;
pub use init_bundle::*;
pub use init_bundle_verifier_page_v2::*;
#[cfg(feature = "global-config")]
pub use init_config::*;
pub use init_config_policy_v2::*;
pub use open_bundle_escrow_v2::*;
pub use place_bid::*;
pub use post_bundle_result_v2::*;
pub use request_job::*;
pub use reveal_bid::*;
pub use select_bundle_verifiers_v2::*;
pub use set_config_policy_v2::*;
pub use submit_job_output::*;
pub use submit_validation::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AuctionInstruction {
    RequestJob = 0,
    PlaceBid = 1,
    EndAuction = 2,
    CloseBid = 3,
    SubmitJobOutput = 4,
    CancelBundle = 5,
    InitBundle = 6,
    SubmitValidation = 7,
    RevealBid = 8,
    CloseRequest = 9,
    AppendData = 10,
    #[cfg(feature = "global-config")]
    InitConfig = 11,
    OpenBundleEscrowV2 = 12,
    CommitAuctionSettlementV2 = 13,
    PostBundleResultV2 = 14,
    FinalizeBundleVerificationV2 = 15,
    ClaimWinnerLstakeV2 = 16,
    ClaimVerifierLstakeV2 = 17,
    ExpireBundleEscrowV2 = 18,
    InitConfigPolicyV2 = 19,
    SetConfigPolicyV2 = 20,
    InitBundleVerifierPageV2 = 21,
    DisputeBundleVerificationV2 = 22,
    SelectBundleVerifiersV2 = 23,
}

/// Wire representation of an IP address inside instruction data and account state.
///
/// This used to be a `#[repr(C)]` Rust enum with two variants and a hand-written
/// `unsafe impl Pod for IpAddr {}`. That impl was unsound. `Pod` is a promise that **every** bit
/// pattern of the type's size is a valid value, and a Rust enum has exactly as many valid tag values
/// as it has variants — here two, out of the 2^32 the tag field can hold. `PlaceBidArgs` is parsed
/// straight out of attacker-controlled instruction data with
/// `bytemuck::try_pod_read_unaligned` (see `macros.rs`), so a bidder could submit a tag of, say, 7 and
/// materialize an enum value that does not exist. Every later `match` on it — including the
/// `From<IpAddr> for std::net::IpAddr` conversion below and the endpoint validation in the listener —
/// would then be undefined behaviour, not merely a wrong answer: the generated jump has no arm to
/// land on.
///
/// The fix keeps the byte layout identical while making the type genuinely `Pod`: an explicit `u32`
/// tag followed by a 16-byte payload, both of which accept every bit pattern. Size (20) and alignment
/// (4) are unchanged, so the offsets of every surrounding field in `PlaceBidArgs` and `Bid` are
/// unchanged, and the payload encoding is preserved exactly:
///
/// * IPv4 stored the four octets in order at payload offset 0. Unchanged; the remaining twelve bytes
///   are now explicitly zeroed rather than left as whatever the enum's union padding happened to hold,
///   which also removes a small uninitialised-memory disclosure from the old writer.
/// * IPv6 stored `Ipv6Addr::segments()` as eight native-endian (little-endian on every Solana target)
///   `u16`s. [`IpAddr::v6`] reproduces that byte for byte via `u16::to_le_bytes`.
///
/// An unrecognised tag is no longer undefined behaviour; it is simply not a valid address.
/// [`IpAddr::to_std`] reports it as `None`, and the infallible `From` conversion degrades to
/// `0.0.0.0`, which the listener's `validate_inference_endpoint` already rejects, so the failure mode
/// is closed rather than exploitable.
#[derive(Clone, Copy, Zeroable, Pod, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// Round-trip through `std::net::IpAddr` for serde so the optional `serde`/`decoder` feature keeps
// emitting a human-readable address string instead of the raw tag/payload pair.
#[cfg_attr(feature = "serde", serde(into = "net::IpAddr", from = "net::IpAddr"))]
#[repr(C)]
pub struct IpAddr {
    /// [`IpAddr::TAG_V4`] or [`IpAddr::TAG_V6`]. Any other value is not a valid address.
    tag: u32,
    /// IPv4: octets in `payload[..4]`, remaining bytes zero.
    /// IPv6: the eight `u16` segments, each little-endian, in order.
    payload: [u8; 16],
}

impl IpAddr {
    /// Tag value for an IPv4 address. Matches the discriminant the previous enum assigned to `V4`.
    pub const TAG_V4: u32 = 0;
    /// Tag value for an IPv6 address. Matches the discriminant the previous enum assigned to `V6`.
    pub const TAG_V6: u32 = 1;

    /// Builds the IPv4 form. Replaces the former `IpAddr::V4(octets)` constructor.
    pub const fn v4(octets: [u8; 4]) -> Self {
        let mut payload = [0u8; 16];
        payload[0] = octets[0];
        payload[1] = octets[1];
        payload[2] = octets[2];
        payload[3] = octets[3];
        Self {
            tag: Self::TAG_V4,
            payload,
        }
    }

    /// Builds the IPv6 form from the same `[u16; 8]` segment array `Ipv6Addr::segments` returns.
    /// Replaces the former `IpAddr::V6(segments)` constructor.
    pub fn v6(segments: [u16; 8]) -> Self {
        let mut payload = [0u8; 16];
        for (chunk, segment) in payload.chunks_exact_mut(2).zip(segments) {
            chunk.copy_from_slice(&segment.to_le_bytes());
        }
        Self {
            tag: Self::TAG_V6,
            payload,
        }
    }

    /// The address as a `std::net::IpAddr`, or `None` when the tag is not one this program wrote.
    ///
    /// Prefer this over the infallible `From` conversion wherever a corrupt or hostile account can be
    /// distinguished from a genuine address.
    pub fn to_std(self) -> Option<net::IpAddr> {
        match self.tag {
            Self::TAG_V4 => Some(net::IpAddr::V4(net::Ipv4Addr::new(
                self.payload[0],
                self.payload[1],
                self.payload[2],
                self.payload[3],
            ))),
            Self::TAG_V6 => {
                let mut segments = [0u16; 8];
                for (segment, chunk) in segments.iter_mut().zip(self.payload.chunks_exact(2)) {
                    // `chunks_exact(2)` yields exactly two bytes, so the conversion cannot fail.
                    *segment = u16::from_le_bytes([chunk[0], chunk[1]]);
                }
                Some(net::IpAddr::V6(net::Ipv6Addr::new(
                    segments[0],
                    segments[1],
                    segments[2],
                    segments[3],
                    segments[4],
                    segments[5],
                    segments[6],
                    segments[7],
                )))
            }
            _ => None,
        }
    }
}

impl Default for IpAddr {
    fn default() -> Self {
        Self::v4([0, 0, 0, 0])
    }
}

impl From<IpAddr> for net::IpAddr {
    fn from(value: IpAddr) -> Self {
        // An unrecognised tag degrades to the unspecified address rather than panicking or inventing a
        // plausible one: `0.0.0.0` is rejected by every endpoint validator in this workspace, so a
        // corrupt account fails closed. Callers that need to tell the two apart use `to_std`.
        value
            .to_std()
            .unwrap_or(net::IpAddr::V4(net::Ipv4Addr::UNSPECIFIED))
    }
}

impl From<net::IpAddr> for IpAddr {
    fn from(value: net::IpAddr) -> Self {
        match value {
            net::IpAddr::V4(ip) => Self::v4(ip.octets()),
            net::IpAddr::V6(ip) => Self::v6(ip.segments()),
        }
    }
}

pub trait InstructionBytes: Pod {
    const INSTRUCTION: AuctionInstruction;
    fn to_bytes(&self) -> Vec<u8> {
        [
            vec![Self::INSTRUCTION.into()],
            bytemuck::bytes_of::<Self>(self).to_vec(),
        ]
        .concat()
    }
}
pub trait InstructionData<'a>: InstructionBytes + TryFrom<&'a [u8]> {}
pub trait InstructionAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;
    fn iter_owned(&'a self) -> impl Iterator<Item = T>
    where
        T: Clone + 'a,
    {
        self.iter().cloned()
    }
}

impl_instruction_data!(
    RequestJobArgs => RequestJob,
    PlaceBidArgs => PlaceBid,
    EndAuctionArgs => EndAuction,
    CloseBidArgs => CloseBid,
    SubmitJobOutputArgs => SubmitJobOutput,
    CancelBundleArgs => CancelBundle,
    InitBundleArgs => InitBundle,
    SubmitValidationArgs => SubmitValidation,
    RevealBidArgs => RevealBid,
    CloseRequestArgs => CloseRequest,
    AppendDataArgs => AppendData,
    OpenBundleEscrowV2Args => OpenBundleEscrowV2,
    CommitAuctionSettlementV2Args => CommitAuctionSettlementV2,
    PostBundleResultV2Args => PostBundleResultV2,
    FinalizeBundleVerificationV2Args => FinalizeBundleVerificationV2,
    ClaimWinnerLstakeV2Args => ClaimWinnerLstakeV2,
    ClaimVerifierLstakeV2Args => ClaimVerifierLstakeV2,
    ExpireBundleEscrowV2Args => ExpireBundleEscrowV2,
    InitConfigPolicyV2Args => InitConfigPolicyV2,
    SetConfigPolicyV2Args => SetConfigPolicyV2,
    InitBundleVerifierPageV2Args => InitBundleVerifierPageV2,
    DisputeBundleVerificationV2Args => DisputeBundleVerificationV2,
    SelectBundleVerifiersV2Args => SelectBundleVerifiersV2,
);

#[cfg(feature = "global-config")]
impl_instruction_data!(InitConfigArgs => InitConfig);
