use num_enum::FromPrimitive;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, FromPrimitive)]
#[repr(u32)]
pub enum AuctionError {
    #[default]
    /// An unknown/unexpected error occurred
    Unknown = 0,
    /// The auction was in an unexpected state. This is likely a bug!
    UnexpectedState = 1,
    /// The request was in an unexpected state.
    UnexpectedRequestState = 2,
    /// The bid account was in an unexpected state.
    UnexpectedBidState = 3,
    /// The auction account is not expired
    AuctionNotExpired = 4,
    /// The balance in the auction account is incorrect.
    IncorrectBalance = 5,
    /// Attempted to close an auction account with a non-zero balance.
    NonZeroBalance = 6,
    /// Error communicating with Solana RPC client
    SolanaRpc = 7,
    /// Attempt to fetch an account failed
    AccountNotFound = 8,
    /// A programmer bug was encountered
    Bug = 9,
    /// An invalid account pubkey was found
    InvalidAccountId = 10,
    /// An incorrect auction account was found
    IncorrectAuction = 11,
    /// Invalid auction status found for the instruction
    InvalidAuctionStatus = 12,
    /// The auction has been expired
    AuctionIsExpired = 13,
    /// Invalid request account
    InvalidRequestId = 14,
    /// The RequestBundle had an invalid state
    InvalidRequestBundleState = 16,
    /// Appending a new JobRequest to an existing bundle failed
    UnableToAddNewJobReqToBundle = 17,
    /// A new bundle was not created, but there are more jobs in a bundle than allowed
    TooManyJobsInBundle = 18,
    /// Decoding the RequestBundle state failed.
    DecodeRequestBundleFailed = 19,
    ///The pubkey provided for the next child bundle was wrong.
    IncorrectChildBundlePubkey = 20,
    ///The pubkey provided for the next child auction was wrong.
    IncorrectChildAuctionPubkey = 21,
    /// Invalid bundle status found for the instruction
    InvalidBundleStatus = 22,
    /// The bundle account is not expired
    BundleNotExpired = 23,
    /// The RequestBundle had an invalid state
    InvalidChildRequestBundleState = 24,
    /// The verifier pubkey has not been assigned to the job/bundle
    VerifierNotAssigned = 25,
    /// No bids were found on auction
    NoBidsFound = 26,
    /// Not enough child bundles were supplied
    NotEnoughBundleAuctionAccounts = 27,
    /// All supplied bundles were unable to fit the request
    FailedToFindAValidBundle = 28,
    /// The bundle is not auctioned
    BundleNotAuctioned = 29,
    /// An invalid bundle registry was found
    InvalidRegistry = 30,
    /// The data account metadata is invalid
    InvalidMetadata = 31,
    /// The current latest Bundle can not get closed
    LatestBundleCanceled = 32,
}

impl Display for AuctionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Auction Error code: {}", self.code())
    }
}

impl AuctionError {
    pub fn code(&self) -> u32 {
        *self as u32
    }
}
