/// The maximum length of an IPFS CID (Content Identifier).
pub const MAX_IPFS_CID_LENGTH: usize = 96;

/// Number of bytes in a pubkey
pub const PUBKEY_BYTES: usize = 32;

/// Constant used as the last seed while deriving a PDA with findProgramAddress
pub const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";
pub const REQUEST_BUNDLE_SEED: &[u8] = b"request_bundle";
pub const JOB_REQUEST_SEED: &[u8] = b"job_request";
pub const BUNDLE_REGISTRY_SEED: &[u8] = b"bundle_registry";
pub const BID_SEED: &[u8] = b"bid";
pub const AUCTION_SEED: &[u8] = b"auction";
pub const CONFIG_SEED: &[u8] = b"global_config";
pub const BUNDLE_ESCROW_V2_SEED: &[u8] = b"bundle_escrow_v2";
/// The minimum number of bundle-auction pairs
/// Eg. if set to 2 means two bundle-auction pairs have to be submitted ie 4 accounts
#[cfg(not(feature = "global-config"))]
pub const MINIMUM_BUNDLE_AUCTION_PAIRS: usize = 2;
/// Base58 auction program ID used by this build.
pub const PROGRAM_ID_B58: &str = env!(
    "AMBIENT_AUCTION_PROGRAM_ID",
    "AMBIENT_AUCTION_PROGRAM_ID must be set when building ambient-auction-api"
);
/// Auction Program ID
pub const ID: [u8; PUBKEY_BYTES] = five8_const::decode_32_const(PROGRAM_ID_B58);

pub const VERIFIERS_PER_AUCTION: usize = 3;
pub const V2_VERIFIER_QUORUM: usize = 2;
