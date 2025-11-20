use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// CloseRequest instruction
///
/// Closes a [`JobRequest`] account.
/// Additionally, closes an associated [`RequestBundle`] account if it is empty.
///
/// # Account References:
///
/// 0. `[WRITE, SIGNER]` Request authority
/// 1. `[WRITE]` Job request account
/// 2. `[WRITE]` Bundle payer account
/// 2. `[WRITE]` Bundle account
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CloseRequestAccounts<'a, T> {
    pub request_authority: &'a T,
    pub job_request: &'a T,
    pub bundle_payer: &'a T,
    pub bundle: &'a T,
    pub registry: &'a T,
    pub auction: &'a T,
    pub auction_payer: &'a T,
    pub child_bundle: &'a T,
    pub child_auction: &'a T,
    pub child_bundle_payer: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for CloseRequestAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [request_authority, job_request, bundle_payer, bundle, registry, auction, auction_payer, child_bundle, child_auction, child_bundle_payer, ..] =
            accounts
        else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            request_authority,
            job_request,
            bundle_payer,
            bundle,
            registry,
            child_auction,
            child_bundle,
            auction,
            auction_payer,
            child_bundle_payer,
        })
    }
}
impl<'a, T> InstructionAccounts<'a, T> for CloseRequestAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.request_authority)
            .chain(std::iter::once(self.job_request))
            .chain(std::iter::once(self.bundle_payer))
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.registry))
            .chain(std::iter::once(self.auction))
            .chain(std::iter::once(self.auction_payer))
            .chain(std::iter::once(self.child_bundle))
            .chain(std::iter::once(self.child_auction))
            .chain(std::iter::once(self.child_bundle_payer))
    }
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CloseRequestArgs {
    pub new_bundle_lamports: u64,
    pub new_auction_lamports: u64,
    pub new_bundle_bump: u64,
}
