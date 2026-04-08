use ambient_auction_api::{VerificationVerdictV2, FINALIZE_BUNDLE_VERIFICATION_V2_DOMAIN};

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
