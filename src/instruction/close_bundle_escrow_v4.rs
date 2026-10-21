use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// Close a settled V4 bundle escrow and return its rent to the account that paid for it.
///
/// `payer` is the account that funded `OpenBundleEscrowV4` / `OpenPricedBundleEscrowV4`. The escrow address is the
/// PDA `[b"bundle_escrow_v2", payer, bundle_hash, bundle_version]`, so the program checks the payer by re-deriving
/// that address from the escrow's own fields. The payer must sign, and it receives the escrow's lamports.
///
/// The program refuses unless the escrow is terminal (finalized or expired), its claim window has passed, every
/// verifier page is closed (`CloseBundleVerifierPageV4` first), and it holds no lamports beyond its rent-exempt
/// minimum.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct CloseBundleEscrowV4Accounts<'a, T> {
    pub payer: &'a T,
    pub bundle_escrow: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for CloseBundleEscrowV4Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, bundle_escrow, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };

        Ok(Self {
            payer,
            bundle_escrow,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for CloseBundleEscrowV4Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer).chain(std::iter::once(self.bundle_escrow))
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct CloseBundleEscrowV4Args {}
