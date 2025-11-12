use crate::{InstructionAccounts, Pubkey, RequestTier};
use bytemuck::{Pod, Zeroable};

/// CancelBundle instruction
///
/// Marks a [`RequestBundle`] account as canceled.
///
/// # Account References:
///
/// 0. `[WRITE]` Parent bundle account

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CancelBundleAccounts<'a, T> {
    pub payer: &'a T,
    pub bundle: &'a T,
    pub child_bundle: &'a T,
    pub registry: &'a T,
    pub system_program: &'a T,
}
impl<'a, T> InstructionAccounts<'a, T> for CancelBundleAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.child_bundle))
            .chain(std::iter::once(self.registry))
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
pub struct CancelBundleArgs {
    pub parent_bundle_key: Pubkey,
    pub bundle_bump: u64,
    pub child_bundle_bump: u64,
    pub context_length_tier: RequestTier,
    pub expiry_duration_tier: RequestTier,
    pub bundle_lamports: u64,
}
