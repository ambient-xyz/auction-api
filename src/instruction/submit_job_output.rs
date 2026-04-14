use crate::error::AuctionError;
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

impl<'a, T> TryFrom<&'a [T]> for SubmitJobOutputAccounts<'a, T> {
    type Error = AuctionError;
    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bid_authority, bundle, job_request, bid, auction, output_data_account, ..] = accounts
        else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            bid_authority,
            bundle,
            job_request,
            bid,
            auction,
            output_data_account,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitJobOutputAccountKeys<T> {
    pub bid_authority: T,
    pub bundle: T,
    pub job_request: T,
    pub bid: T,
    pub auction: T,
    pub output_data_account: T,
}

impl<T> SubmitJobOutputAccountKeys<T> {
    pub fn as_accounts(&self) -> SubmitJobOutputAccounts<'_, T> {
        SubmitJobOutputAccounts {
            bid_authority: &self.bid_authority,
            bundle: &self.bundle,
            job_request: &self.job_request,
            bid: &self.bid,
            auction: &self.auction,
            output_data_account: &self.output_data_account,
        }
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SubmitJobOutputAccountKeys<T>
where
    T: 'a,
{
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(&self.bid_authority)
            .chain(std::iter::once(&self.bundle))
            .chain(std::iter::once(&self.job_request))
            .chain(std::iter::once(&self.bid))
            .chain(std::iter::once(&self.auction))
            .chain(std::iter::once(&self.output_data_account))
    }
}

impl<'a, T> SubmitJobOutputAccounts<'a, T>
where
    T: Clone,
{
    pub fn to_account_keys(&self) -> SubmitJobOutputAccountKeys<T> {
        SubmitJobOutputAccountKeys {
            bid_authority: self.bid_authority.clone(),
            bundle: self.bundle.clone(),
            job_request: self.job_request.clone(),
            bid: self.bid.clone(),
            auction: self.auction.clone(),
            output_data_account: self.output_data_account.clone(),
        }
    }
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
