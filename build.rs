use std::{env, fs, path::PathBuf};

const DEFAULT_PROGRAM_ID_B58: &str = "Auction111111111111111111111111111111111111";
const PROGRAM_ID_OVERRIDE_ENV: &str = "AMBIENT_AUCTION_PROGRAM_ID";

fn main() {
    println!("cargo:rerun-if-env-changed={PROGRAM_ID_OVERRIDE_ENV}");

    let program_id = env::var(PROGRAM_ID_OVERRIDE_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_PROGRAM_ID_B58.to_string());

    validate_program_id(&program_id);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let generated = format!(
        "/// Base58 auction program ID used by this build.\n\
         pub const PROGRAM_ID_B58: &str = \"{program_id}\";\n\
         /// Auction Program ID\n\
         pub const ID: [u8; PUBKEY_BYTES] = five8_const::decode_32_const(PROGRAM_ID_B58);\n"
    );

    fs::write(out_dir.join("program_id.rs"), generated).expect("failed to write program_id.rs");
}

fn validate_program_id(program_id: &str) {
    let decoded = bs58::decode(program_id)
        .into_vec()
        .unwrap_or_else(|error| panic!("invalid {PROGRAM_ID_OVERRIDE_ENV} value `{program_id}`: {error}"));

    assert!(
        decoded.len() == 32,
        "{PROGRAM_ID_OVERRIDE_ENV} must decode to 32 bytes, got {}",
        decoded.len()
    );
}
