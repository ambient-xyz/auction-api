use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::net;

mod append_data;
mod cancel_bundle;
mod close_bid;
mod close_request;
mod end_auction;
mod init_bundle;
mod init_config;
mod place_bid;
mod request_job;
mod reveal_bid;
mod submit_job_output;
mod submit_validation;

use crate::macros::impl_instruction_data;
pub use append_data::*;
pub use cancel_bundle::*;
pub use close_bid::*;
pub use close_request::*;
pub use end_auction::*;
pub use init_bundle::*;
pub use init_config::*;
pub use place_bid::*;
pub use request_job::*;
pub use reveal_bid::*;
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
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone;
}

#[cfg(feature = "global-config")]
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
    InitConfigArgs => InitConfig,
);
#[cfg(not(feature = "global-config"))]
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
);
