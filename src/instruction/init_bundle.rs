use crate::{InstructionAccounts, RequestTier};
use bytemuck::{Pod, Zeroable};

/// InitBundle instruction
///
/// Creates a [`RequestBundle`] account.
///
/// # Account References:
///
/// 0. `[WRITE, SIGNER]` Funding account
/// 1. `[WRITE]` New bundle account
/// 2. `[WRITE]` Bundle registry account
/// 3. `[READ]` System program
#[derive(Debug, Clone)]
#[repr(C)]
pub struct InitBundleAccounts<'a, T> {
    pub payer: &'a T,
    pub bundle: &'a T,
    pub registry: &'a T,
    pub system_program: &'a T,
    pub auction: &'a T,
}
impl<'a, T> InstructionAccounts<'a, T> for InitBundleAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.registry))
            .chain(std::iter::once(self.system_program))
            .chain(std::iter::once(self.auction))
    }
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Debug)]
#[repr(C)]
pub struct InitBundleArgs {
    /// Context length tier type
    pub context_length_tier: RequestTier,
    /// Expiry duration tier type
    pub expiry_duration_tier: RequestTier,
    pub bundle_lamports: u64,
    pub registry_lamports: u64,
    pub bundle_bump: u64,
    pub registry_bump: u64,
}
