#![cfg(feature = "decoder")]

use ambient_auction_api::{AccountLayoutVersion, BundleEscrowV2};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn bundle_escrow_decoder_includes_versioned_pricing() {
    for (version, has_prices) in [
        (AccountLayoutVersion::V5, true),
        (AccountLayoutVersion::V5, false),
        (AccountLayoutVersion::V2, false),
    ] {
        let mut bytes = vec![0; BundleEscrowV2::account_len(version)];
        assert!(BundleEscrowV2::default().write_bytes_with_layout(&mut bytes, version));
        if has_prices {
            let mut escrow = BundleEscrowV2::from_bytes_mut(&mut bytes).unwrap();
            let pricing = escrow.pricing_mut().unwrap();
            pricing.pricing_posted_page_bitmap = 5;
            pricing.pricing_sealed = 1;
            pricing.pricing_commitment = [9; 32];
        }

        let mut child = Command::new(env!("CARGO_BIN_EXE_decode-account"))
            .arg("bundle-escrow-v2")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let decoded: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let expected = if version == AccountLayoutVersion::V5 {
            serde_json::json!({
                "pricing_posted_page_bitmap": if has_prices { 5 } else { 0 },
                "pricing_sealed": u8::from(has_prices),
                "_reserved0": [0, 0, 0, 0, 0, 0],
                "pricing_commitment": vec![if has_prices { 9 } else { 0 }; 32],
            })
        } else {
            serde_json::Value::Null
        };
        assert_eq!(decoded.get("pricing"), Some(&expected));
    }
}
