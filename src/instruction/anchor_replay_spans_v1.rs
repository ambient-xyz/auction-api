use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// Anchor the SHA-256 of an engine's latest span commit (exact replay). The miner's node key signs; `payer` funds
/// the anchor account on the first anchor.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct AnchorReplaySpansV1Accounts<'a, T> {
    pub miner_node: &'a T,
    pub payer: &'a T,
    pub replay_anchor: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for AnchorReplaySpansV1Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [miner_node, payer, replay_anchor, system_program, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            miner_node,
            payer,
            replay_anchor,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for AnchorReplaySpansV1Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.miner_node)
            .chain(std::iter::once(self.payer))
            .chain(std::iter::once(self.replay_anchor))
            .chain(std::iter::once(self.system_program))
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct AnchorReplaySpansV1Args {
    /// SHA-256 of the engine id; a seed of the anchor account.
    pub engine: [u8; 32],
    /// The span whose commit is anchored. It must be past the latest anchored span.
    pub span_id: u64,
    /// SHA-256 of that span's signed commit.
    pub commit_sha256: [u8; 32],
}
