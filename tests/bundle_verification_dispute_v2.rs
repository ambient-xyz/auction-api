use ambient_auction_api::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, BundleVerificationDisputeV2,
    BundleVerificationDisputeV2Kind, Pubkey, VerificationVerdictV2,
};

fn test_pubkey(byte: u8) -> Pubkey {
    [byte; 32].into()
}

#[test]
fn bundle_verification_dispute_v2_kind_round_trips() {
    assert_eq!(
        BundleVerificationDisputeV2Kind::try_from(1),
        Ok(BundleVerificationDisputeV2Kind::MissedVerification)
    );
    assert_eq!(
        BundleVerificationDisputeV2Kind::try_from(2),
        Ok(BundleVerificationDisputeV2Kind::PaidVerdictDispute)
    );
    assert_eq!(BundleVerificationDisputeV2Kind::try_from(9), Err(9));
}

#[test]
fn bundle_verification_dispute_v2_bytes_round_trip() {
    assert_eq!(BundleVerificationDisputeV2::PAYLOAD_LEN, 184);
    assert_eq!(BundleVerificationDisputeV2::LEN, 192);
    let dispute = BundleVerificationDisputeV2 {
        bundle_escrow: test_pubkey(1),
        kind: BundleVerificationDisputeV2Kind::PaidVerdictDispute,
        original_verdict: VerificationVerdictV2::Verified,
        replacement_verifier_count: 3,
        replacement_verifier_quorum: 2,
        replacement_verifiers: [test_pubkey(2), test_pubkey(3), test_pubkey(4)],
        replacement_deadline_slot: 55,
        bond_lamports: 89,
        bond_refund_recipient: test_pubkey(5),
        ..Default::default()
    };
    let mut bytes = vec![0u8; BundleVerificationDisputeV2::LEN];

    assert!(dispute.write_bytes(&mut bytes));
    assert_eq!(
        &bytes[..AccountHeaderV1::LEN],
        bytemuck::bytes_of(&AccountHeaderV1::new(
            AccountDiscriminator::BundleVerificationDisputeV2
        ))
    );

    let parsed = BundleVerificationDisputeV2::from_bytes(&bytes).unwrap();
    assert_eq!(*parsed.as_raw(), dispute);
}

#[test]
fn legacy_zeroed_snapshot_fields_decode_as_zero() {
    let dispute = BundleVerificationDisputeV2 {
        replacement_verifier_count: 3,
        replacement_verifier_quorum: 2,
        ..Default::default()
    };
    let mut bytes = vec![0u8; BundleVerificationDisputeV2::LEN];
    assert!(dispute.write_bytes(&mut bytes));

    let count_offset = AccountHeaderV1::LEN + 35;
    bytes[count_offset..count_offset + 2].fill(0);

    let parsed = BundleVerificationDisputeV2::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.replacement_verifier_count, 0);
    assert_eq!(parsed.replacement_verifier_quorum, 0);
}

#[test]
fn bundle_verification_dispute_v2_rejects_wrong_header() {
    let dispute = BundleVerificationDisputeV2::default();
    let mut bytes = vec![0u8; BundleVerificationDisputeV2::LEN];
    assert!(dispute.write_bytes(&mut bytes));

    bytes[0] = AccountDiscriminator::BundleVerifierPageV2 as u8;
    assert!(BundleVerificationDisputeV2::from_bytes(&bytes).is_none());

    bytes[0] = AccountDiscriminator::BundleVerificationDisputeV2 as u8;
    bytes[1] = AccountLayoutVersion::V2 as u8;
    assert!(BundleVerificationDisputeV2::from_bytes(&bytes).is_none());
}

#[test]
fn dispute_v5_preserves_the_historical_payload_and_checks_exact_layouts() {
    let dispute = BundleVerificationDisputeV2 {
        bond_lamports: 89,
        replacement_deadline_slot: 55,
        ..Default::default()
    };
    let mut bytes = vec![0; BundleVerificationDisputeV2::LEN_V5];
    assert!(dispute.write_bytes_with_layout(&mut bytes, AccountLayoutVersion::V5));
    {
        let mut parsed = BundleVerificationDisputeV2::from_bytes_mut(&mut bytes).unwrap();
        let evidence = parsed.v5_mut().unwrap();
        evidence.authorized = 1;
        evidence.verification_hash = [3; 32];
        evidence.page_hashes[0] = [4; 32];
    }
    let parsed = BundleVerificationDisputeV2::from_bytes(&bytes).unwrap();
    assert_eq!(*parsed.as_raw(), dispute);
    assert_eq!(parsed.v5().unwrap().page_hashes[0], [4; 32]);
    assert!(
        BundleVerificationDisputeV2::from_bytes(&bytes[..BundleVerificationDisputeV2::LEN])
            .is_none()
    );
    bytes[1] = AccountLayoutVersion::V1 as u8;
    assert!(BundleVerificationDisputeV2::from_bytes(&bytes).is_none());
    let historical =
        BundleVerificationDisputeV2::from_bytes(&bytes[..BundleVerificationDisputeV2::LEN])
            .unwrap();
    assert_eq!(*historical.as_raw(), dispute);
    assert!(historical.v5().is_none());
}

#[test]
fn dispute_manifest_has_a_distinct_fixed_signing_contract() {
    use ambient_auction_api::BundleDisputeEvidenceV5Message;
    let message = BundleDisputeEvidenceV5Message::new(
        [1; 32],
        0x0807060504030201,
        0x1817161514131211,
        2,
        [3; 32],
        [[4; 32], [5; 32], [0; 32]],
    );
    let expected = [
        b"ambient.dispute.pages.v1\0\0\0\0\0\0\0\0".as_slice(),
        &[1; 32],
        &[1, 2, 3, 4, 5, 6, 7, 8],
        &[17, 18, 19, 20, 21, 22, 23, 24],
        &[3; 32],
        &[2, 0, 0, 0, 0, 0, 0, 0],
        &[4; 32],
        &[5; 32],
        &[0; 32],
    ]
    .concat();
    assert_eq!(message.to_bytes(), expected);
}
