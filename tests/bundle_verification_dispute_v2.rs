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
