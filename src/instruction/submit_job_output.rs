use crate::InstructionAccounts;
use bytemuck::{Pod, Zeroable};

/// SubmitJobOutput instruction
///
/// Submits the output of running a [`JobRequest`] account.
///
/// # Account References:
///
/// 0. `[SIGNER]` Bid authority
/// 1. `[WRITE]` Bundle account
/// 2. `[WRITE]` Job request account
/// 3. `[READ]` Bid account
/// 4. `[READ]` Auction account
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SubmitJobOutputAccounts<'a, T> {
    pub bid_authority: &'a T,
    pub bundle: &'a T,
    pub job_request: &'a T,
    pub bid: &'a T,
    pub auction: &'a T,
    pub output_data_account: &'a T,
}
impl<'a, T> InstructionAccounts<'a, T> for SubmitJobOutputAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bid_authority)
            .chain(std::iter::once(self.bundle))
            .chain(std::iter::once(self.job_request))
            .chain(std::iter::once(self.bid))
            .chain(std::iter::once(self.auction))
            .chain(std::iter::once(self.output_data_account))
    }
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct SubmitJobOutputArgs {
    pub output_token_count: u64,
    pub input_token_count: u64,
    pub merkle_root: [u8; 32],
    pub output_hash: [u8; 32],
    // All zeroes if no encryption is used
    pub merkle_root_iv: [u8; 16],
    pub output_hash_iv: [u8; 16],
    pub encryption_node_publickey: [u8; 32],
}
