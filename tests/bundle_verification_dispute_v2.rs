use ambient_auction_api::{
    AccountDiscriminator, AccountHeaderV1, BundleVerificationDisputeV2,
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
    let dispute = BundleVerificationDisputeV2 {
        bundle_escrow: test_pubkey(1),
        kind: BundleVerificationDisputeV2Kind::PaidVerdictDispute,
        original_verdict: VerificationVerdictV2::Verified,
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
