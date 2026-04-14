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
mod end_auction;
mod expire_bundle_escrow_v2;
mod finalize_bundle_verification_v2;
mod init_bundle;
#[cfg(feature = "global-config")]
mod init_config;
mod init_config_policy_v2;
mod open_bundle_escrow_v2;
mod place_bid;
mod post_bundle_result_v2;
mod request_job;
mod reveal_bid;
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
pub use end_auction::*;
pub use expire_bundle_escrow_v2::*;
pub use finalize_bundle_verification_v2::*;
pub use init_bundle::*;
#[cfg(feature = "global-config")]
pub use init_config::*;
pub use init_config_policy_v2::*;
pub use open_bundle_escrow_v2::*;
pub use place_bid::*;
pub use post_bundle_result_v2::*;
pub use request_job::*;
pub use reveal_bid::*;
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
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum IpAddr {
    // Padding
    V4([u8; 4]),
    V6([u16; 8]),
}
unsafe impl Pod for IpAddr {}

impl Default for IpAddr {
    fn default() -> Self {
        Self::V4([0, 0, 0, 0])
    }
}

impl From<IpAddr> for net::IpAddr {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(b) => net::IpAddr::V4(net::Ipv4Addr::new(b[0], b[1], b[2], b[3])),
            IpAddr::V6(b) => net::IpAddr::V6(net::Ipv6Addr::new(
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            )),
        }
    }
}

impl From<net::IpAddr> for IpAddr {
    fn from(value: net::IpAddr) -> Self {
        match value {
            net::IpAddr::V4(ip) => IpAddr::V4(ip.octets()),
            net::IpAddr::V6(ip) => IpAddr::V6(ip.segments()),
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
);

#[cfg(feature = "global-config")]
impl_instruction_data!(InitConfigArgs => InitConfig);
