use crate::error::AuctionError;
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

impl<'a, T> TryFrom<&'a [T]> for InitConfigAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [payer, config, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };
        Ok(Self {
            payer,
            config,
            system_program,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitConfigAccountKeys<T> {
    pub payer: T,
    pub config: T,
    pub system_program: T,
}

impl<T> InitConfigAccountKeys<T> {
    pub fn as_accounts(&self) -> InitConfigAccounts<'_, T> {
        InitConfigAccounts {
            payer: &self.payer,
            config: &self.config,
            system_program: &self.system_program,
        }
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitConfigAccountKeys<T>
where
    T: 'a,
{
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(&self.payer)
            .chain(std::iter::once(&self.config))
            .chain(std::iter::once(&self.system_program))
    }
}

impl<'a, T> InitConfigAccounts<'a, T>
where
    T: Clone,
{
    pub fn to_account_keys(&self) -> InitConfigAccountKeys<T> {
        InitConfigAccountKeys {
            payer: self.payer.clone(),
            config: self.config.clone(),
            system_program: self.system_program.clone(),
        }
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitConfigAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.config))
            .chain(std::iter::once(self.system_program))
    }
}
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InitConfigArgs {
    pub minimum_bundle_auction_pairs: u64,
    pub update_authority: MaybePubkey,
    pub config_lamports: u64,
}
