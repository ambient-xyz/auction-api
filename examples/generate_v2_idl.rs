use ambient_auction_api::{error::AuctionError, idl::constants::PROGRAM_ID};
use shank_idl::{extract_idl, ParseIdlOpts};
use std::{env, fs, path::PathBuf};

fn main() {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: generate_v2_idl <output.json>");
    let idl = extract_idl(
        "src/lib.rs",
        ParseIdlOpts {
            program_address_override: Some(PROGRAM_ID.to_owned()),
            ..ParseIdlOpts::default()
        },
    )
    .expect("extract Shank IDL")
    .expect("crate contains an IDL");
    let mut value = serde_json::to_value(idl).expect("serialize Shank IDL");
    value["errors"] = (0..)
        .map_while(|code| AuctionError::try_from_code(code).ok())
        .map(|error| {
            serde_json::json!({
                "code": error.code(),
                "name": error.name(),
                "msg": error.message(),
            })
        })
        .collect();

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create IDL output directory");
    }
    let json = serde_json::to_string_pretty(&value).expect("format Shank IDL") + "\n";
    fs::write(output, json).expect("write Shank IDL");
}
