use ambient_auction_api::{
    FinalizeBundleVerificationV2Args, FinalizeBundleVerificationV2Message, VerificationVerdictV2,
    FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN,
};

const FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT: &[u8] = b"ambient.bundle.verify.v2";

#[test]
fn verification_verdict_v2_round_trips_through_raw_values() {
    assert_eq!(
        VerificationVerdictV2::try_from(0),
        Ok(VerificationVerdictV2::Unset)
    );
    assert_eq!(
        VerificationVerdictV2::try_from(2),
        Ok(VerificationVerdictV2::Rejected)
    );
    assert_eq!(u8::from(VerificationVerdictV2::Verified), 1);
    assert_eq!(VerificationVerdictV2::try_from(9), Err(9));
}

#[test]
fn verification_verdict_v2_matches_on_associated_constants() {
    let label = match VerificationVerdictV2::Verified {
        VerificationVerdictV2::Unset => "unset",
        VerificationVerdictV2::Verified => "verified",
        VerificationVerdictV2::Rejected => "rejected",
        _ => "invalid",
    };

    assert_eq!(label, "verified");
}

#[test]
fn finalize_bundle_verification_v2_domain_is_zero_padded() {
    assert_eq!(
        &FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN
            [..FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT.len()],
        FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT,
    );
    assert!(FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN
        [FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN_TEXT.len()..]
        .iter()
        .all(|byte| *byte == 0));
}

#[test]
fn finalize_message_matches_miner_client_lifecycle_vector() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/finalize-v2-lifecycle.json")).unwrap();
    let hash = |name: &str| {
        let hex = fixture[name].as_str().unwrap();
        std::array::from_fn(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap())
    };
    let deadline = fixture["settlementDeadlineSlot"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let mut message = FinalizeBundleVerificationV2Message::new(
        [1; 32],
        fixture["bundleVersion"]
            .as_u64()
            .unwrap()
            .try_into()
            .unwrap(),
        hash("bundleHashHex"),
        hash("auctionHashHex"),
        hash("resultHashHex"),
        hash("verificationHashHex"),
        VerificationVerdictV2::Verified,
        fixture["acceptedOutputTokens"].as_u64().unwrap(),
        fixture["winnerPayoutLamports"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
        deadline,
    );
    let bytes = message.to_bytes();
    assert_eq!(bytes.len(), 232);
    assert_eq!(
        memoffset::offset_of!(
            FinalizeBundleVerificationV2Message,
            settlement_deadline_slot
        ),
        224
    );
    assert_eq!(std::mem::size_of::<FinalizeBundleVerificationV2Args>(), 56);
    assert_eq!(&bytes[224..], &deadline.to_le_bytes());
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(hex, fixture["messageHex"].as_str().unwrap());

    message.settlement_deadline_slot += 1;
    let next = message.to_bytes();
    assert_eq!(&bytes[..224], &next[..224]);
    assert_ne!(&bytes[224..], &next[224..]);
}
