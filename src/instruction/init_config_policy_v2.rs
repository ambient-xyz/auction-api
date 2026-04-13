use crate::error::AuctionError;
use crate::{ConfigPolicyV2, InstructionAccounts};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct InitConfigPolicyV2Accounts<'a, T> {
    pub authority: &'a T,
    pub config: &'a T,
    pub config_policy: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for InitConfigPolicyV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [authority, config, config_policy, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            authority,
            config,
            config_policy,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitConfigPolicyV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.authority)
            .chain(std::iter::once(self.config))
            .chain(std::iter::once(self.config_policy))
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
pub struct InitConfigPolicyV2Args {
    pub config_policy_lamports: u64,
    pub policy: ConfigPolicyV2,
}
