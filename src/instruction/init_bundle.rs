use crate::error::AuctionError;
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
}

impl<'a, T> TryFrom<&'a [T]> for InitBundleAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, bundle, registry, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            payer,
            bundle,
            registry,
            system_program,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitBundleAccountKeys<T> {
    pub payer: T,
    pub bundle: T,
    pub registry: T,
    pub system_program: T,
}

impl<T> InitBundleAccountKeys<T> {
    pub fn as_accounts(&self) -> InitBundleAccounts<'_, T> {
        InitBundleAccounts {
            payer: &self.payer,
            bundle: &self.bundle,
            registry: &self.registry,
            system_program: &self.system_program,
        }
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitBundleAccountKeys<T>
where
    T: 'a,
{
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(&self.payer)
            .chain(std::iter::once(&self.bundle))
            .chain(std::iter::once(&self.registry))
            .chain(std::iter::once(&self.system_program))
    }
}

impl<'a, T> InitBundleAccounts<'a, T>
where
    T: Clone,
{
    pub fn to_account_keys(&self) -> InitBundleAccountKeys<T> {
        InitBundleAccountKeys {
            payer: self.payer.clone(),
            bundle: self.bundle.clone(),
            registry: self.registry.clone(),
            system_program: self.system_program.clone(),
        }
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitBundleAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.registry))
            .chain(std::iter::once(self.system_program))
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
