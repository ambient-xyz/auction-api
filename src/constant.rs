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
/// Auction Program ID
pub const ID: [u8; PUBKEY_BYTES] =
    five8_const::decode_32_const("Auction111111111111111111111111111111111111");

pub const VERIFIERS_PER_AUCTION: usize = 3;
