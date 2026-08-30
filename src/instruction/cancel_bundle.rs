use crate::error::AuctionError;
use crate::{InstructionAccounts, Pubkey, RequestTierRaw};
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

impl<'a, T> TryFrom<&'a [T]> for CancelBundleAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, bundle, child_bundle, registry, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            payer,
            bundle,
            child_bundle,
            registry,
            system_program,
        })
    }
}
impl<'a, T> InstructionAccounts<'a, T> for CancelBundleAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.child_bundle))
            .chain(std::iter::once(self.registry))
            .chain(std::iter::once(self.system_program))
    }
}
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CancelBundleArgs {
    pub parent_bundle_key: Pubkey,
    pub bundle_bump: u64,
    pub child_bundle_bump: u64,
    /// Context length tier. Validated raw wrapper: this struct is parsed from attacker-controlled
    /// instruction data with `bytemuck`, so the field type must accept every bit pattern. Convert
    /// with `RequestTier::try_from(args.context_length_tier)`.
    pub context_length_tier: RequestTierRaw,
    /// Expiry duration tier. Validated raw wrapper; see `context_length_tier`.
    pub expiry_duration_tier: RequestTierRaw,
    pub bundle_lamports: u64,
}
