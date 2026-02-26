use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// UpdateVerifier instruction
#[derive(Debug, Clone)]
#[repr(C)]
pub struct UpdateVerifierAccounts<'a, T> {
    pub vote_account: &'a T,
    pub vote_authority: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for UpdateVerifierAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [vote_account, vote_authority, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            vote_account,
            vote_authority,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for UpdateVerifierAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.vote_account).chain(std::iter::once(self.vote_authority))
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
pub struct UpdateVerifierArgs {
    tee_enabled: BoolPOD,
}

#[derive(Zeroable, Clone, Copy, Eq, PartialEq, Debug, Pod)]
#[repr(transparent)]
pub struct RawBoolPOD(u64);

impl RawBoolPOD {
    pub fn new(bool: bool) -> Self {
        Self::from(bool)
    }
}

impl From<BoolPOD> for RawBoolPOD {
    fn from(value: BoolPOD) -> Self {
        Self(u64::from(value))
    }
}

impl From<bool> for RawBoolPOD {
    fn from(value: bool) -> Self {
        Self(value as u64)
    }
}

#[derive(Zeroable, Clone, Copy, Eq, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u64)]
pub enum BoolPOD {
    False = 0,
    True = 1,
}

impl TryFrom<RawBoolPOD> for BoolPOD {
    type Error = AuctionError;
    fn try_from(value: RawBoolPOD) -> Result<Self, Self::Error> {
        BoolPOD::try_from(value.0).map_err(|_| Self::Error::InvalidBoolOption)
    }
}
