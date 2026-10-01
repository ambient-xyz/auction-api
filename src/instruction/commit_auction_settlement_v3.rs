use crate::{CommitAuctionSettlementV2Accounts, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};

pub type CommitAuctionSettlementV3Accounts<'a, T> = CommitAuctionSettlementV2Accounts<'a, T>;

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct CommitAuctionSettlementV3Args {
    pub auction_hash: [u8; 32],
    pub winner_node_pubkey: [u8; PUBKEY_BYTES],
}
