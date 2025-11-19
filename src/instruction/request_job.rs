use crate::{InstructionAccounts, MaybePubkey, PUBKEY_BYTES};
use bytemuck::{Pod, Zeroable};

/// RequestJob instruction
///
/// creates a [`JobRequest`] account after placing it in a bundle.
/// Additionally, creates an associated [`Auction`] and a child [`RequestBundle`] account if the bundle is filled.
///
/// # Account References:
///
/// 0. `[WRITE, SIGNER]` Funding account
/// 1. `[WRITE]` New job request account
/// 2. `[WRITE]` Bundle registry account
/// 3. `[READ]` System program
/// 4. `[WRITE]` Input data account
/// 5. `[WRITE]` Parent bundle account
/// 6. `[WRITE]` Parent auction account
/// 7. `[WRITE]` Child bundle account
/// 8. `[WRITE]` Child auction account
///
///     Repeating (0 or more):
///
///       `[WRITE]` Additional bundle-auction account pair(s)
/// 9. `[WRITE]` Last bundle account
#[derive(Clone, Debug)]
#[repr(C)]
pub struct RequestJobAccounts<'a, T, U> {
    pub payer: &'a T,
    pub job_request: &'a T,
    pub registry: &'a T,
    pub input_data: &'a T,
    pub system_program: &'a T,
    pub config: &'a T,
    pub bundle_auction_account_pairs: U,
    pub last_bundle: &'a T,
}

impl<'a, T, U> InstructionAccounts<'a, T> for RequestJobAccounts<'a, T, U>
where
    U: AsRef<[T]>,
{
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.payer)
            .chain(std::iter::once(self.job_request))
            .chain(std::iter::once(self.registry))
            .chain(std::iter::once(self.input_data))
            .chain(std::iter::once(self.system_program))
            .chain(std::iter::once(self.config))
            .chain(self.bundle_auction_account_pairs.as_ref())
            .chain(std::iter::once(self.last_bundle))
    }
    fn iter_owned(&self) -> impl Iterator<Item = T>
    where
        T: Clone,
    {
        self.iter().cloned()
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct RequestJobArgs {
    /// The maximum price per output token
    pub max_price_per_output_token: u64,
    /// The maximum output token this request accepts
    pub max_output_tokens: u64,
    /// The authority of the request account
    pub authority: [u8; PUBKEY_BYTES],
    /// IPFS CID for inputs of the job.
    pub input_hash: [u8; 32],
    pub input_hash_iv: [u8; 16],
    /// The user seed used to derive the job request PDA
    pub job_request_seed: [u8; 32],
    /// Starting lamports for the bundle account initialization
    pub new_bundle_lamports: u64,
    /// Number of input tokens in the request
    pub input_tokens: u64,
    /// Bump used to derive the request account
    pub bump: u64,
    /// lamports to initialize the new auction with
    pub new_auction_lamports: u64,
    pub input_data_account: MaybePubkey,
}
