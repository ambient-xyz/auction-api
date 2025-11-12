use crate::constant::PUBKEY_BYTES;
use crate::InstructionAccounts;
use bytemuck::Pod;
use bytemuck::Zeroable;
use std::num::NonZeroU64;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AppendDataAccounts<'a, T> {
    pub data_authority: &'a T,
    pub data_account: &'a T,
    pub system_program: &'a T,
}
impl<'a, T> InstructionAccounts<'a, T> for AppendDataAccounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.data_authority)
            .chain(std::iter::once(self.data_account))
            .chain(std::iter::once(self.system_program))
    }
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}

/// Expected header in the beginning of the AppendData instruction
#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Debug)]
#[repr(C)]
pub struct AppendDataArgs {
    /// Offset to append data from (inclusive).
    ///
    /// NOTE: has to be greater than `size_of::<Metadata>()`
    pub offset: u64,
    /// Seed used to create the data_account.
    /// Must match seed used in `CreateAccountWithSeed` system program instruction.
    pub seed: [u8; PUBKEY_BYTES],
    pub seed_len: u64,
    /// Length of decompressed data. `None` if no compression is used
    pub decompressed_data_length: Option<NonZeroU64>,
}
