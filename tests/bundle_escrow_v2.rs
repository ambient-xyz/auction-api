use ambient_auction_api::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, BundleEscrowV2,
    BundleEscrowV2Status, InvalidBundleEscrowV2Transition, Pubkey, RequestTier,
    VERIFIERS_PER_AUCTION,
};

fn test_pubkey(byte: u8) -> Pubkey {
    [byte; 32].into()
}

#[test]
fn bundle_escrow_v2_status_round_trips_through_raw_values() {
    assert_eq!(
        BundleEscrowV2Status::try_from(0),
        Ok(BundleEscrowV2Status::Open)
    );
    assert_eq!(
        BundleEscrowV2Status::try_from(5),
        Ok(BundleEscrowV2Status::Expired)
    );
    assert_eq!(u64::from(BundleEscrowV2Status::Awarded), 1);
    assert_eq!(BundleEscrowV2Status::try_from(99), Err(99));
}

#[test]
fn bundle_escrow_v2_status_identifies_terminal_states() {
    assert!(!BundleEscrowV2Status::Open.is_terminal());
    assert!(!BundleEscrowV2Status::Awarded.is_terminal());
    assert!(BundleEscrowV2Status::FinalizedVerified.is_terminal());
    assert!(BundleEscrowV2Status::FinalizedRejected.is_terminal());
    assert!(BundleEscrowV2Status::Expired.is_terminal());
}

#[test]
fn bundle_escrow_v2_status_matches_on_associated_constants() {
    let label = match BundleEscrowV2Status::ResultPosted {
        BundleEscrowV2Status::Open => "open",
        BundleEscrowV2Status::Awarded => "awarded",
        BundleEscrowV2Status::ResultPosted => "result-posted",
        BundleEscrowV2Status::FinalizedVerified => "finalized-verified",
        BundleEscrowV2Status::FinalizedRejected => "finalized-rejected",
        BundleEscrowV2Status::Expired => "expired",
        _ => "invalid",
    };

    assert_eq!(label, "result-posted");
}

#[test]
fn bundle_escrow_v2_v1_bytes_round_trip() {
    let bundle = BundleEscrowV2 {
        status: BundleEscrowV2Status::Awarded,
        reward_tier: RequestTier::Standard,
        bundle_version: 7,
        total_input_tokens: 11,
        ..Default::default()
    };
    let mut bytes = vec![0u8; BundleEscrowV2::LEN];

    assert!(bundle.write_v1_bytes(&mut bytes));
    assert_eq!(
        &bytes[..AccountHeaderV1::LEN],
        bytemuck::bytes_of(&AccountHeaderV1::new(AccountDiscriminator::BundleEscrowV2))
    );

    let parsed = BundleEscrowV2::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.status, BundleEscrowV2Status::Awarded);
    assert_eq!(parsed.bundle_version, 7);

    let copied = BundleEscrowV2::read(&bytes).unwrap();
    assert_eq!(copied, bundle);
}

#[test]
fn bundle_escrow_v2_rejects_wrong_discriminator() {
    let bundle = BundleEscrowV2::default();
    let mut bytes = vec![0u8; BundleEscrowV2::LEN];
    assert!(bundle.write_v1_bytes(&mut bytes));
    bytes[0] = AccountDiscriminator::Bundle as u8;

    assert!(BundleEscrowV2::from_bytes(&bytes).is_none());
}

#[test]
fn bundle_escrow_v2_rejects_wrong_version() {
    let bundle = BundleEscrowV2::default();
    let mut bytes = vec![0u8; BundleEscrowV2::LEN];
    assert!(bundle.write_v1_bytes(&mut bytes));
    bytes[1] = AccountLayoutVersion::LegacyV0 as u8;

    assert!(BundleEscrowV2::from_bytes(&bytes).is_none());
}

#[test]
fn bundle_escrow_v2_rejects_wrong_length() {
    let bytes = vec![0u8; BundleEscrowV2::LEN - 1];
    assert!(BundleEscrowV2::from_bytes(&bytes).is_none());
}

#[test]
fn bundle_version_is_not_layout_version() {
    let bundle = BundleEscrowV2 {
        bundle_version: 99,
        ..Default::default()
    };
    let mut bytes = vec![0u8; BundleEscrowV2::LEN];
    assert!(bundle.write_v1_bytes(&mut bytes));

    let parsed = BundleEscrowV2::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.bundle_version, 99);
    assert_eq!(parsed.header().version, AccountLayoutVersion::V1 as u8);
}

#[test]
fn bundle_escrow_v2_award_updates_coupled_fields() {
    let mut bundle = BundleEscrowV2::default();
    let selected_verifiers = [test_pubkey(4), test_pubkey(5), test_pubkey(6)];

    bundle
        .award(
            [7; 32],
            test_pubkey(1),
            test_pubkey(2),
            42,
            selected_verifiers,
        )
        .unwrap();

    assert_eq!(bundle.status, BundleEscrowV2Status::Awarded);
    assert_eq!(bundle.auction_hash, [7; 32]);
    assert_eq!(bundle.winner_node_pubkey, test_pubkey(1));
    assert_eq!(bundle.winner_vote_account, test_pubkey(2));
    assert_eq!(bundle.clearing_price_per_output_token, 42);
    assert_eq!(bundle.selected_verifiers, selected_verifiers);
}

#[test]
fn bundle_escrow_v2_post_result_updates_coupled_fields() {
    let mut bundle = BundleEscrowV2 {
        status: BundleEscrowV2Status::Awarded,
        ..Default::default()
    };

    bundle.post_result([8; 32], 55).unwrap();

    assert_eq!(bundle.status, BundleEscrowV2Status::ResultPosted);
    assert_eq!(bundle.result_hash, [8; 32]);
    assert_eq!(bundle.posted_output_tokens, 55);
}

#[test]
fn bundle_escrow_v2_finalize_verified_updates_coupled_fields() {
    let mut bundle = BundleEscrowV2 {
        status: BundleEscrowV2Status::ResultPosted,
        ..Default::default()
    };

    bundle
        .finalize(
            BundleEscrowV2Status::FinalizedVerified,
            [9; 32],
            21,
            0b011,
            2,
            [5, 8, 0],
        )
        .unwrap();

    assert_eq!(bundle.status, BundleEscrowV2Status::FinalizedVerified);
    assert_eq!(bundle.verification_hash, [9; 32]);
    assert_eq!(bundle.accepted_output_tokens, 21);
    assert_eq!(bundle.verifier_page_count, 2);
    assert_eq!(bundle.verifier_reward_remaining, [5, 8, 0]);
    assert_eq!(bundle.quorum_verifier_bitmap, 0b011);
}

#[test]
fn bundle_escrow_v2_finalize_rejected_updates_coupled_fields() {
    let mut bundle = BundleEscrowV2 {
        status: BundleEscrowV2Status::ResultPosted,
        ..Default::default()
    };

    bundle
        .finalize(
            BundleEscrowV2Status::FinalizedRejected,
            [10; 32],
            0,
            0b101,
            0,
            [0; VERIFIERS_PER_AUCTION],
        )
        .unwrap();

    assert_eq!(bundle.status, BundleEscrowV2Status::FinalizedRejected);
    assert_eq!(bundle.verification_hash, [10; 32]);
    assert_eq!(bundle.accepted_output_tokens, 0);
    assert_eq!(bundle.quorum_verifier_bitmap, 0b101);
}

#[test]
fn bundle_escrow_v2_expire_accepts_each_preterminal_state() {
    for status in [
        BundleEscrowV2Status::Open,
        BundleEscrowV2Status::Awarded,
        BundleEscrowV2Status::ResultPosted,
    ] {
        let mut bundle = BundleEscrowV2 {
            status,
            ..Default::default()
        };
        bundle.expire().unwrap();
        assert_eq!(bundle.status, BundleEscrowV2Status::Expired);
    }
}

#[test]
fn bundle_escrow_v2_rejects_invalid_transitions() {
    let mut awarded = BundleEscrowV2 {
        status: BundleEscrowV2Status::Awarded,
        ..Default::default()
    };
    assert_eq!(
        awarded
            .award(
                [1; 32],
                test_pubkey(1),
                test_pubkey(2),
                1,
                [test_pubkey(3); 3]
            )
            .unwrap_err(),
        InvalidBundleEscrowV2Transition {
            from: BundleEscrowV2Status::Awarded,
            to: BundleEscrowV2Status::Awarded,
        }
    );

    let mut open = BundleEscrowV2::default();
    assert_eq!(
        open.post_result([2; 32], 5).unwrap_err(),
        InvalidBundleEscrowV2Transition {
            from: BundleEscrowV2Status::Open,
            to: BundleEscrowV2Status::ResultPosted,
        }
    );

    let mut result_posted = BundleEscrowV2 {
        status: BundleEscrowV2Status::ResultPosted,
        ..Default::default()
    };
    assert_eq!(
        result_posted
            .award(
                [3; 32],
                test_pubkey(1),
                test_pubkey(2),
                1,
                [test_pubkey(3); 3]
            )
            .unwrap_err(),
        InvalidBundleEscrowV2Transition {
            from: BundleEscrowV2Status::ResultPosted,
            to: BundleEscrowV2Status::Awarded,
        }
    );

    let mut finalized = BundleEscrowV2 {
        status: BundleEscrowV2Status::FinalizedVerified,
        ..Default::default()
    };
    assert_eq!(
        finalized.expire().unwrap_err(),
        InvalidBundleEscrowV2Transition {
            from: BundleEscrowV2Status::FinalizedVerified,
            to: BundleEscrowV2Status::Expired,
        }
    );
    assert_eq!(
        finalized
            .finalize(
                BundleEscrowV2Status::FinalizedRejected,
                [4; 32],
                0,
                0,
                0,
                [0; VERIFIERS_PER_AUCTION],
            )
            .unwrap_err(),
        InvalidBundleEscrowV2Transition {
            from: BundleEscrowV2Status::FinalizedVerified,
            to: BundleEscrowV2Status::FinalizedRejected,
        }
    );
}
