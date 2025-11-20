use crate::error::AuctionError;
use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// SubmitValidation instruction
///
/// Submits verification result of a [`RequestBundle`] account.
///
/// # Account References:
///
/// 0. `[WRITE]` Bundle account
/// 1. `[WRITE]` Vote account
/// 2. `[READ]` Vote program
/// 3. `[Signer]` Vote authority
/// 4. `[WRITE]` Job request account
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SubmitValidationAccounts<'a, T> {
    pub bundle: &'a T,
    pub vote_account: &'a T,
    pub vote_program: &'a T,
    pub vote_authority: &'a T,
    pub job_request: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for SubmitValidationAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle, vote_account, vote_program, vote_authority, job_request, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            bundle,
            vote_account,
            vote_program,
            vote_authority,
            job_request,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SubmitValidationAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle)
            .chain(std::iter::once(self.vote_account))
            .chain(std::iter::once(self.vote_program))
            .chain(std::iter::once(self.vote_authority))
            .chain(std::iter::once(self.job_request))
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
pub struct SubmitValidationArgs {
    pub num_successes: u64,
    pub num_failures: u64,
}
