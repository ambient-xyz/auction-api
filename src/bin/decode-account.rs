#![cfg(feature = "decoder")]
use std::io::{self, Read as _};

use ambient_auction_api::{
    instruction::SubmitJobOutputArgs, Auction, Bid, JobRequest, RequestBundle, VerificationState,
};
use base64::Engine as _;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

fn display_job_request(buffer: Vec<u8>) -> Result<(), String> {
    eprintln!("Expected len: {}", JobRequest::LEN);
    let JobRequest {
        bundle,
        max_output_tokens,
        input_hash,
        output_token_count,
        input_token_count,
        status,
        verification,
        ..
    } = bytemuck::try_pod_read_unaligned::<JobRequest>(&buffer).map_err(|e| {
        format!("To decode JobRequest from account bytes. Is it the right account type? {e}")
    })?;

    let VerificationState {
        merkle_root,
        verified_tokens,
        assigned_verifiers,
        verifier_states,
        output_hash,
        ..
    } = verification;
    let input_hash_b64 = base64::prelude::BASE64_STANDARD.encode(input_hash);
    let output_hash_b64 = base64::prelude::BASE64_STANDARD.encode(output_hash);
    let merkle_root_b64 = base64::prelude::BASE64_STANDARD.encode(merkle_root);
    let bundle_b58 = bs58::encode(bundle).into_string();
    let assigned_verifiers_s = assigned_verifiers
        .map(|v| bs58::encode(v).into_string())
        .join(", ");
    let verification_states_s = verifier_states.map(|v| v.to_string()).join(", ");
    eprintln!(
        "bundle: {bundle_b58}
input hash: {input_hash_b64}
max output tokens: {max_output_tokens}
output token count: {output_token_count}
input token count: {input_token_count}
job status: {status}
merkle root: {merkle_root_b64}
status: {status}
output hash: {output_hash_b64}
verified tokens: {verified_tokens:?}
assigned verifiers: [{assigned_verifiers_s}]
verification states: [{verification_states_s}]"
    );
    Ok(())
}

fn display_submit_job_output(buffer: Vec<u8>) -> Result<(), String> {
    eprintln!("Expected len: {}", size_of::<SubmitJobOutputArgs>());
    let SubmitJobOutputArgs {
        output_token_count,
        input_token_count,
        merkle_root,
        output_hash,
        output_hash_iv,
        merkle_root_iv,
        encryption_node_publickey,
    } = bytemuck::try_pod_read_unaligned(&buffer).map_err(|e| {
        format!("To decode SubmitJobOutput from transaction bytes. Is it the right data type? {e}")
    })?;
    let output_hash_b64 = base64::prelude::BASE64_STANDARD.encode(output_hash);
    let merkle_root_b64 = base64::prelude::BASE64_STANDARD.encode(merkle_root);
    eprintln!(
        "input token count: {input_token_count}
output token count: {output_token_count}
merkle root: {merkle_root_b64}
output hash: {output_hash_b64}
output hash iv: {output_hash_iv:?}
merkle root iv: {merkle_root_iv:?}
encryption node public key: {encryption_node_publickey:?}"
    );
    Ok(())
}

fn display_generic<T: bytemuck::Pod + Serialize>(buffer: Vec<u8>) -> Result<(), String> {
    let data = bytemuck::try_pod_read_unaligned::<T>(&buffer)
        .map_err(|e| format!("To decode from transaction bytes. Is it the right data type? {e}"))?;
    println!("{}", serde_json::to_string_pretty(&data).unwrap());
    Ok(())
}

#[derive(Parser)]
struct Cli {
    ///Whether to treat input as base64 encoded text before passing it to the struct decoder
    #[arg(short = 'd', long, value_enum, default_value_t = Default::default())]
    decoder: Decoder,
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Default, Debug)]
enum Decoder {
    #[default]
    Raw,
    Base64,
    Base58,
    Hex,
}

#[derive(Subcommand)]
enum Commands {
    /// Decode a `JobRequest` struct from raw data
    JobRequest,
    /// Decode a `Auction` struct from raw data
    Auction,
    /// Decode the transaction input data from a submit_job_output transaction
    SubmitJobOutput,
    Bid,
    Bundle,
}

fn parse_string(buf: Vec<u8>) -> Result<String, String> {
    String::from_utf8(buf)
        .map_err(|e| format!("Unable to decode into utf-8, are you sure it's base64 encoded? {e}"))
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let mut buffer = Vec::new(); // Create a buffer to hold the input
                                 // Read from stdin
    io::stdin()
        .read_to_end(&mut buffer)
        .expect("To read account bytes from stdin");

    match cli.decoder {
        Decoder::Raw => {}
        Decoder::Base64 => {
            //... as a string
            let s = parse_string(buffer)?;
            // Then as a base64 encoded string
            buffer = base64::engine::general_purpose::STANDARD
                .decode(s)
                .map_err(|e| format!("Unable to decode as base64: {e}"))?;
        }
        Decoder::Base58 => {
            //... as a string
            let s = parse_string(buffer)?;
            // Then as a base58 encoded string
            buffer = bs58::decode(s)
                .into_vec()
                .map_err(|e| format!("Unable to decode as base58: {e}"))?;
        }
        Decoder::Hex => {
            //... as a string
            let s = parse_string(buffer)?;
            buffer = hex::decode(s).map_err(|e| format!("Unable to decode as hexadecimal: {e}"))?;
        }
    }
    eprintln!("Data len: {}", buffer.len());

    match cli.command {
        Commands::JobRequest => display_job_request(buffer),
        Commands::Auction => display_generic::<Auction>(buffer),
        Commands::SubmitJobOutput => display_submit_job_output(buffer[1..].to_vec()),
        Commands::Bid => display_generic::<Bid>(buffer),
        Commands::Bundle => display_generic::<RequestBundle>(buffer),
    }
}
