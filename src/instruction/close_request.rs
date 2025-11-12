use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// CloseRequest instruction
///
/// Closes a [`JobRequest`] account.
/// Additionally, closes an associated [`RequestBundle`] account if it is empty.
///
/// # Account References:
///
/// 0. `[WRITE, SIGNER]` Request authority
/// 1. `[WRITE]` Job request account
/// 2. `[WRITE]` Bundle payer account
/// 2. `[WRITE]` Bundle account
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CloseRequestAccounts<'a, T> {
    pub request_authority: &'a T,
    pub job_request: &'a T,
    pub bundle_payer: &'a T,
    pub bundle: &'a T,
}

impl<'a, T> InstructionAccounts<'a, T> for CloseRequestAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.request_authority)
            .chain(std::iter::once(self.job_request))
            .chain(std::iter::once(self.bundle_payer))
            .chain(std::iter::once(self.bundle))
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
pub struct CloseRequestArgs {}
