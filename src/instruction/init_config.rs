use crate::{InstructionAccounts, MaybePubkey};
use bytemuck::{Pod, Zeroable};

/// Initializes the [`Config`] account.
///
/// # Account References:
///
/// 1. `[WRITE]` Funding account
/// 1. `[WRITE]` Config account to be created
#[derive(Debug, Clone)]
#[repr(C)]
pub struct InitConfigAccounts<'a, T> {
    pub payer: &'a T,
    pub config: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> InstructionAccounts<'a, T> for InitConfigAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.config))
            .chain(std::iter::once(self.system_program))
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
pub struct InitConfigArgs {
    pub minimum_bundle_auction_pairs: u64,
    pub update_authority: MaybePubkey,
    pub config_lamports: u64,
}
