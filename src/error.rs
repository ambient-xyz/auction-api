use num_enum::TryFromPrimitive;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, TryFromPrimitive)]
#[repr(u32)]
pub enum AuctionError {
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
    /// Not enough accounts were supplied
    NotEnoughAccounts = 33,
    /// Invalid Config Account data was found
    InvalidConfigData = 34,
    /// Illegal config account found
    IllegalConfigOwner = 35,
    /// Found an Invalid value for JobVerificationState
    InvalidJobVerificationState = 36,
    /// The token range is already verified
    AlreadyVerified = 37,
    /// The bundle escrow v2 account had an invalid state
    InvalidBundleEscrowV2State = 38,
    /// Invalid bundle escrow v2 status found for the instruction
    InvalidBundleEscrowV2Status = 39,
    /// The provided coordinator does not match the stored coordinator
    InvalidCoordinator = 40,
    /// The provided vote account does not match the expected node identity
    InvalidVoteAccount = 41,
    /// The winner can not be selected as a verifier
    WinnerCannotBeVerifier = 42,
    /// Invalid verifier quorum provided for the bundle escrow v2 flow
    InvalidVerifierQuorum = 43,
    /// Missing or malformed Ed25519 verification instruction
    InvalidEd25519Instruction = 44,
    /// The requested reward has already been claimed
    RewardAlreadyClaimed = 45,
    /// The bundle escrow v2 result has not been posted yet
    ResultNotPosted = 46,
    /// The bundle escrow v2 settlement has not been posted yet
    SettlementNotCommitted = 47,
    /// The bundle escrow v2 has not reached the required deadline
    DeadlineNotReached = 48,
    /// The bundle escrow v2 claim window is still open
    ClaimWindowStillOpen = 49,
    /// The accepted output token count exceeds the posted output token count
    AcceptedOutputExceedsPosted = 50,
    /// The posted output token count exceeds the allowed maximum
    PostedOutputExceedsMax = 51,
    /// The authority is not allowed to post this result
    UnauthorizedResultPoster = 52,
    /// The bundle escrow v2 payout exceeds the escrow balance
    InsufficientEscrowBalance = 53,
    /// The bundle verifier page v2 account had an invalid state
    InvalidBundleVerifierPageV2State = 54,
    /// Invalid config policy v2 account data was found
    InvalidConfigPolicyV2Data = 55,
    /// Illegal config policy v2 account found
    IllegalConfigPolicyV2Owner = 56,
    /// The signer is not allowed to update config policy v2
    UnauthorizedConfigPolicyAuthority = 57,
    /// The configured account layout version is invalid or unsupported
    InvalidAccountLayoutVersion = 58,
    /// The configured account layout version is not allowed for this authority
    UnauthorizedAccountLayoutVersion = 59,
    /// Invalid verifier count configured for the v2 flow
    InvalidVerifierCount = 60,
    /// Invalid tier config configured for the v2 flow
    InvalidTierConfig = 61,
    /// The bundle escrow v2 settlement deadline has passed
    SettlementDeadlinePassed = 62,
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

    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::UnexpectedState => "UnexpectedState",
            Self::UnexpectedRequestState => "UnexpectedRequestState",
            Self::UnexpectedBidState => "UnexpectedBidState",
            Self::AuctionNotExpired => "AuctionNotExpired",
            Self::IncorrectBalance => "IncorrectBalance",
            Self::NonZeroBalance => "NonZeroBalance",
            Self::SolanaRpc => "SolanaRpc",
            Self::AccountNotFound => "AccountNotFound",
            Self::Bug => "Bug",
            Self::InvalidAccountId => "InvalidAccountId",
            Self::IncorrectAuction => "IncorrectAuction",
            Self::InvalidAuctionStatus => "InvalidAuctionStatus",
            Self::AuctionIsExpired => "AuctionIsExpired",
            Self::InvalidRequestId => "InvalidRequestId",
            Self::InvalidRequestBundleState => "InvalidRequestBundleState",
            Self::UnableToAddNewJobReqToBundle => "UnableToAddNewJobReqToBundle",
            Self::TooManyJobsInBundle => "TooManyJobsInBundle",
            Self::DecodeRequestBundleFailed => "DecodeRequestBundleFailed",
            Self::IncorrectChildBundlePubkey => "IncorrectChildBundlePubkey",
            Self::IncorrectChildAuctionPubkey => "IncorrectChildAuctionPubkey",
            Self::InvalidBundleStatus => "InvalidBundleStatus",
            Self::BundleNotExpired => "BundleNotExpired",
            Self::InvalidChildRequestBundleState => "InvalidChildRequestBundleState",
            Self::VerifierNotAssigned => "VerifierNotAssigned",
            Self::NoBidsFound => "NoBidsFound",
            Self::NotEnoughBundleAuctionAccounts => "NotEnoughBundleAuctionAccounts",
            Self::FailedToFindAValidBundle => "FailedToFindAValidBundle",
            Self::BundleNotAuctioned => "BundleNotAuctioned",
            Self::InvalidRegistry => "InvalidRegistry",
            Self::InvalidMetadata => "InvalidMetadata",
            Self::LatestBundleCanceled => "LatestBundleCanceled",
            Self::NotEnoughAccounts => "NotEnoughAccounts",
            Self::InvalidConfigData => "InvalidConfigData",
            Self::IllegalConfigOwner => "IllegalConfigOwner",
            Self::InvalidJobVerificationState => "InvalidJobVerificationState",
            Self::AlreadyVerified => "AlreadyVerified",
            Self::InvalidBundleEscrowV2State => "InvalidBundleEscrowV2State",
            Self::InvalidBundleEscrowV2Status => "InvalidBundleEscrowV2Status",
            Self::InvalidCoordinator => "InvalidCoordinator",
            Self::InvalidVoteAccount => "InvalidVoteAccount",
            Self::WinnerCannotBeVerifier => "WinnerCannotBeVerifier",
            Self::InvalidVerifierQuorum => "InvalidVerifierQuorum",
            Self::InvalidEd25519Instruction => "InvalidEd25519Instruction",
            Self::RewardAlreadyClaimed => "RewardAlreadyClaimed",
            Self::ResultNotPosted => "ResultNotPosted",
            Self::SettlementNotCommitted => "SettlementNotCommitted",
            Self::DeadlineNotReached => "DeadlineNotReached",
            Self::ClaimWindowStillOpen => "ClaimWindowStillOpen",
            Self::AcceptedOutputExceedsPosted => "AcceptedOutputExceedsPosted",
            Self::PostedOutputExceedsMax => "PostedOutputExceedsMax",
            Self::UnauthorizedResultPoster => "UnauthorizedResultPoster",
            Self::InsufficientEscrowBalance => "InsufficientEscrowBalance",
            Self::InvalidBundleVerifierPageV2State => "InvalidBundleVerifierPageV2State",
            Self::InvalidConfigPolicyV2Data => "InvalidConfigPolicyV2Data",
            Self::IllegalConfigPolicyV2Owner => "IllegalConfigPolicyV2Owner",
            Self::UnauthorizedConfigPolicyAuthority => "UnauthorizedConfigPolicyAuthority",
            Self::InvalidAccountLayoutVersion => "InvalidAccountLayoutVersion",
            Self::UnauthorizedAccountLayoutVersion => "UnauthorizedAccountLayoutVersion",
            Self::InvalidVerifierCount => "InvalidVerifierCount",
            Self::InvalidTierConfig => "InvalidTierConfig",
            Self::SettlementDeadlinePassed => "SettlementDeadlinePassed",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown auction error",
            Self::UnexpectedState => "Auction state is invalid for this operation",
            Self::UnexpectedRequestState => "Request state is invalid for this operation",
            Self::UnexpectedBidState => "Bid state is invalid for this operation",
            Self::AuctionNotExpired => "Auction has not expired",
            Self::IncorrectBalance => "Auction balance is incorrect",
            Self::NonZeroBalance => "Account balance must be zero to close",
            Self::SolanaRpc => "Solana RPC request failed",
            Self::AccountNotFound => "Account was not found",
            Self::Bug => "Internal auction bug encountered",
            Self::InvalidAccountId => "Account pubkey is invalid",
            Self::IncorrectAuction => "Auction account does not match request",
            Self::InvalidAuctionStatus => "Auction status is invalid for this instruction",
            Self::AuctionIsExpired => "Auction has expired",
            Self::InvalidRequestId => "Request account is invalid",
            Self::InvalidRequestBundleState => "Request bundle state is invalid",
            Self::UnableToAddNewJobReqToBundle => "Failed to append job request to bundle",
            Self::TooManyJobsInBundle => "Bundle already has the maximum number of jobs",
            Self::DecodeRequestBundleFailed => "Failed to decode request bundle",
            Self::IncorrectChildBundlePubkey => "Child bundle pubkey is incorrect",
            Self::IncorrectChildAuctionPubkey => "Child auction pubkey is incorrect",
            Self::InvalidBundleStatus => "Bundle status is invalid for this instruction",
            Self::BundleNotExpired => "Bundle has not expired",
            Self::InvalidChildRequestBundleState => "Child request bundle state is invalid",
            Self::VerifierNotAssigned => "Verifier is not assigned to this bundle",
            Self::NoBidsFound => "No bids were found",
            Self::NotEnoughBundleAuctionAccounts => {
                "Not enough bundle auction accounts were provided"
            }
            Self::FailedToFindAValidBundle => "No supplied bundle can accept the request",
            Self::BundleNotAuctioned => "Bundle is not auctioned",
            Self::InvalidRegistry => "Bundle registry is invalid",
            Self::InvalidMetadata => "Metadata account is invalid",
            Self::LatestBundleCanceled => "Latest bundle cannot be canceled",
            Self::NotEnoughAccounts => "Not enough accounts were provided",
            Self::InvalidConfigData => "Config account data is invalid",
            Self::IllegalConfigOwner => "Config account owner is invalid",
            Self::InvalidJobVerificationState => "Job verification state is invalid",
            Self::AlreadyVerified => "Token range is already verified",
            Self::InvalidBundleEscrowV2State => "Bundle escrow v2 state is invalid",
            Self::InvalidBundleEscrowV2Status => {
                "Bundle escrow v2 status is invalid for this instruction"
            }
            Self::InvalidCoordinator => "Coordinator does not match bundle escrow",
            Self::InvalidVoteAccount => "Vote account does not match the expected node identity",
            Self::WinnerCannotBeVerifier => "Winner cannot be selected as a verifier",
            Self::InvalidVerifierQuorum => "Verifier quorum is invalid",
            Self::InvalidEd25519Instruction => "Ed25519 verification instruction is invalid",
            Self::RewardAlreadyClaimed => "Reward has already been claimed",
            Self::ResultNotPosted => "Bundle escrow v2 result has not been posted",
            Self::SettlementNotCommitted => "Bundle escrow v2 settlement has not been committed",
            Self::DeadlineNotReached => "Bundle escrow v2 deadline has not been reached",
            Self::ClaimWindowStillOpen => "Bundle escrow v2 claim window is still open",
            Self::AcceptedOutputExceedsPosted => {
                "Accepted output tokens exceed posted output tokens"
            }
            Self::PostedOutputExceedsMax => "Posted output tokens exceed the allowed maximum",
            Self::UnauthorizedResultPoster => "Authority is not allowed to post this result",
            Self::InsufficientEscrowBalance => "Escrow balance is insufficient for payout",
            Self::InvalidBundleVerifierPageV2State => "Bundle verifier page v2 state is invalid",
            Self::InvalidConfigPolicyV2Data => "Config policy v2 account data is invalid",
            Self::IllegalConfigPolicyV2Owner => "Config policy v2 account owner is invalid",
            Self::UnauthorizedConfigPolicyAuthority => {
                "Authority is not allowed to update config policy v2"
            }
            Self::InvalidAccountLayoutVersion => {
                "Configured account layout version is invalid or unsupported"
            }
            Self::UnauthorizedAccountLayoutVersion => {
                "Authority is not allowed to use this account layout version"
            }
            Self::InvalidVerifierCount => "Verifier count is invalid",
            Self::InvalidTierConfig => "Tier config is invalid",
            Self::SettlementDeadlinePassed => "Settlement deadline has passed",
        }
    }

    pub fn try_from_code(code: u32) -> Result<Self, u32> {
        Self::try_from(code).map_err(|_| code)
    }

    pub fn describe_code(code: u32) -> Option<&'static str> {
        Self::try_from_code(code).ok().map(|error| error.message())
    }
}
