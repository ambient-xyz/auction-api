use crate::error::AuctionError;
use crate::{ConfigPolicyV2, InstructionAccounts};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SetConfigPolicyV2Accounts<'a, T> {
    pub authority: &'a T,
    pub config_policy: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for SetConfigPolicyV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [authority, config_policy, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            authority,
            config_policy,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SetConfigPolicyV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.authority).chain(std::iter::once(self.config_policy))
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
pub struct SetConfigPolicyV2Args {
    pub policy: ConfigPolicyV2,
}
