use crate::{MaybePubkey, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};
use std::num::NonZeroU64;

#[derive(Clone, Copy, Zeroable, Debug, PartialEq, Pod)]
#[repr(C)]
pub struct Metadata {
    /// None if no compression is used
    pub decompressed_len: Option<NonZeroU64>,
    pub job_request_key: MaybePubkey,
    /// Seed used to create the data_account.
    /// Must match seed used in `CreateAccountWithSeed` system program instruction.
    pub seed: [u8; PUBKEY_BYTES],
    pub seed_len: u64,
    /// length of the text stored in the account
    pub payload_len: u64,
}

impl Metadata {
    pub const LEN: usize = std::mem::size_of::<Metadata>();
}
