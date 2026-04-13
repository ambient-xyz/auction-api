use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// UpdateVerifier instruction
#[derive(Debug, Clone)]
#[repr(C)]
pub struct UpdateVerifierAccounts<'a, T> {
    pub vote_account: &'a T,
    pub vote_authority: &'a T,
    pub auction_verifiers: &'a T,
    #[cfg(feature = "global-config")]
    pub config: &'a T,
}

#[cfg(not(feature = "global-config"))]
impl<'a, T> TryFrom<&'a [T]> for UpdateVerifierAccounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [vote_account, vote_authority, auction_verifiers, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            vote_account,
            vote_authority,
            auction_verifiers,
        })
    }
}

#[cfg(feature = "global-config")]
impl<'a, T> TryFrom<&'a [T]> for UpdateVerifierAccounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [vote_account, vote_authority, auction_verifiers, config, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            vote_account,
            vote_authority,
            auction_verifiers,
            config,
        })
    }
}

#[cfg(not(feature = "global-config"))]
impl<'a, T> InstructionAccounts<'a, T> for UpdateVerifierAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.vote_account)
            .chain(std::iter::once(self.vote_authority))
            .chain(std::iter::once(self.auction_verifiers))
    }

    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}

#[cfg(feature = "global-config")]
impl<'a, T> InstructionAccounts<'a, T> for UpdateVerifierAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.vote_account)
            .chain(std::iter::once(self.vote_authority))
            .chain(std::iter::once(self.auction_verifiers))
            .chain(std::iter::once(self.config))
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
    tee_enabled: RawBoolPOD,
}

impl UpdateVerifierArgs {
    pub fn new(tee_enabled: bool) -> Self {
        Self {
            tee_enabled: RawBoolPOD::from(tee_enabled),
        }
    }

    pub fn tee_enabled(self) -> Result<bool, AuctionError> {
        match self.tee_enabled.0 {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(AuctionError::InvalidBoolOption),
        }
    }
}

#[derive(Zeroable, Clone, Copy, Eq, PartialEq, Debug, Pod)]
#[repr(transparent)]
pub struct RawBoolPOD(u64);

impl RawBoolPOD {
    pub fn new(value: bool) -> Self {
        Self::from(value)
    }
}

impl From<bool> for RawBoolPOD {
    fn from(value: bool) -> Self {
        Self(value as u64)
    }
}
