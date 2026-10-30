use ambient_auction_api::*;
use memoffset::offset_of;

const _: () = assert!(ReplayFaultKind::COUNT <= REPLAY_FAULT_KIND_CAPACITY);

fn policy() -> ReplayPolicyV1 {
    let mut suspension_hours = [0; REPLAY_FAULT_KIND_CAPACITY];
    suspension_hours[ReplayFaultKind::Mismatch as usize] = 168;
    suspension_hours[ReplayFaultKind::FalseFlag as usize] = 24;
    ReplayPolicyV1 {
        class_id: [7; 32],
        version: REPLAY_POLICY_V1_VERSION,
        tier_mask: tier_bit(RequestTier::Large),
        arbiters: 3,
        hold_slots: 9_000,
        forfeit_mask: 1 << ReplayFaultKind::Mismatch as u16,
        suspension_hours,
        ..ReplayPolicyV1::default()
    }
}

#[test]
fn replay_policy_fills_the_former_reserved_words() {
    assert_eq!(ReplayPolicyV1::LEN, 96);
    assert_eq!(std::mem::align_of::<ReplayPolicyV1>(), 8);
    assert_eq!(offset_of!(ReplayPolicyV1, class_id), 0);
    assert_eq!(offset_of!(ReplayPolicyV1, version), 32);
    assert_eq!(offset_of!(ReplayPolicyV1, tier_mask), 33);
    assert_eq!(offset_of!(ReplayPolicyV1, arbiters), 34);
    assert_eq!(offset_of!(ReplayPolicyV1, hold_slots), 40);
    assert_eq!(offset_of!(ReplayPolicyV1, forfeit_mask), 64);
    assert_eq!(offset_of!(ReplayPolicyV1, suspension_hours), 66);
    assert_eq!(offset_of!(ConfigPolicyV2, replay), 1_448);
    assert_eq!(ConfigPolicyV2::LEN, 1_568);
    assert_eq!(
        ConfigPolicyV2::production_default().replay,
        ReplayPolicyV1::default()
    );
}

#[test]
fn replay_policy_validation() {
    assert!(ReplayPolicyV1::default().validate());
    assert!(!ReplayPolicyV1::default().enabled());
    let ok = policy();
    assert!(ok.validate());
    assert!(ok.is_replay_tier(RequestTier::Large));
    assert!(!ok.is_replay_tier(RequestTier::Standard));
    assert!(ok.forfeits(ReplayFaultKind::Mismatch));
    assert!(!ok.forfeits(ReplayFaultKind::FalseFlag));
    assert_eq!(ok.suspension_hours(ReplayFaultKind::FalseFlag), 24);

    let bad: [fn(&mut ReplayPolicyV1); 13] = [
        |p| p.version = 2,
        |p| p.class_id = [0; 32],
        |p| p.tier_mask = 0,
        |p| p.tier_mask = tier_bit(RequestTier::Small),
        |p| p.tier_mask |= 1 << 5,
        |p| p.arbiters = 0,
        |p| p.arbiters = MAX_REPLAY_ARBITERS + 1,
        |p| p._reserved0[4] = 1,
        |p| p.hold_slots = 0,
        |p| p.hold_slots = u64::from(u32::MAX) + 1,
        |p| p._reserved1[0] = 1,
        |p| p.forfeit_mask = 1 << ReplayFaultKind::COUNT,
        |p| p.suspension_hours[ReplayFaultKind::COUNT] = 1,
    ];
    for (case, mutate) in bad.iter().enumerate() {
        let mut p = ok;
        mutate(&mut p);
        assert!(!p.validate(), "case {case} should be invalid");
    }
    // Off means every byte is zero.
    let mut off = ReplayPolicyV1 { version: 0, ..ok };
    assert!(!off.validate());
    off = ReplayPolicyV1::default();
    off.hold_slots = 1;
    assert!(!off.validate());
}

#[test]
fn fault_kinds_round_trip_the_coordinator_names() {
    for kind in ReplayFaultKind::ALL {
        assert_eq!(ReplayFaultKind::from_name(kind.as_str()), Some(kind));
    }
    assert_eq!(ReplayFaultKind::from_name("verifier_timeout"), None);
}

#[test]
fn replay_settings_patch_payload_is_distinct() {
    assert_eq!(std::mem::size_of::<SetConfigPolicyReplayV1Args>(), 104);
    assert_ne!(
        std::mem::size_of::<SetConfigPolicyReplayV1Args>(),
        std::mem::size_of::<SetConfigPolicyV2Args>()
    );
    assert_ne!(
        std::mem::size_of::<SetConfigPolicyReplayV1Args>(),
        std::mem::size_of::<SetConfigPolicySmallV3Args>()
    );
    let args = SetConfigPolicyReplayV1Args {
        patch_kind: ConfigPolicyV2PatchKind::REPLAY_SETTINGS,
        _reserved0: [0; 7],
        replay: policy(),
    };
    let bytes = args.to_bytes();
    assert_eq!(bytes[0], AuctionInstruction::SetConfigPolicyV2 as u8);
    assert_eq!(bytes[1], 8);
    assert_eq!(
        SetConfigPolicyReplayV1Args::try_from(&bytes[1..]).unwrap(),
        args
    );
}

#[test]
fn escrow_lifecycle_carries_the_replay_flag() {
    assert_eq!(offset_of!(BundleEscrowLifecycleV4Data, exact_replay), 2);
    assert_eq!(std::mem::size_of::<BundleEscrowLifecycleV4Data>(), 48);
    assert_eq!(BundleEscrowV2::account_len(AccountLayoutVersion::V4), 656);
}

#[test]
fn replay_anchor_layout_and_ring() {
    assert_eq!(ReplayAnchorV1::LEN, 480);
    assert_eq!(offset_of!(RawReplayAnchorV1Data, engine), 32);
    assert_eq!(offset_of!(RawReplayAnchorV1Data, first_span_id), 64);
    assert_eq!(offset_of!(RawReplayAnchorV1Data, anchor_count), 72);
    assert_eq!(offset_of!(RawReplayAnchorV1Data, bump), 80);
    assert_eq!(offset_of!(RawReplayAnchorV1Data, recent), 88);
    assert_eq!(std::mem::size_of::<ReplayAnchorEntryV1>(), 48);
    assert_eq!(AuctionInstruction::AnchorReplaySpansV1 as u8, 36);
    assert_eq!(std::mem::size_of::<AnchorReplaySpansV1Args>(), 72);

    let mut state = ReplayAnchorV1 {
        miner_node: Pubkey::from([1; 32]),
        engine: [2; 32],
        bump: 254,
        ..ReplayAnchorV1::default()
    };
    assert!(state.latest().is_none());
    for span in 5..5 + 11u64 {
        state.push(ReplayAnchorEntryV1 {
            span_id: span,
            slot: 100 + span,
            commit_sha256: [span as u8; 32],
        });
    }
    assert_eq!(state.first_span_id, 5);
    assert_eq!(state.anchor_count, 11);
    assert_eq!(state.latest().unwrap().span_id, 15);
    let mut kept: Vec<u64> = state.recent.iter().map(|e| e.span_id).collect();
    kept.sort_unstable();
    assert_eq!(kept, (8..=15).collect::<Vec<_>>());

    let mut bytes = vec![0; ReplayAnchorV1::LEN];
    assert!(state.write_bytes(&mut bytes));
    assert_eq!(bytes[0], AccountDiscriminator::ReplayAnchorV1 as u8);
    assert_eq!(bytes[1], AccountLayoutVersion::V1 as u8);
    let read = ReplayAnchorV1::from_bytes(&bytes).unwrap();
    assert_eq!(*read, state);
    // Wrong length, discriminator or version: not an anchor.
    assert!(ReplayAnchorV1::from_bytes(&bytes[..ReplayAnchorV1::LEN - 1]).is_none());
    let mut other = bytes.clone();
    other[0] = AccountDiscriminator::BundleEscrowV2 as u8;
    assert!(ReplayAnchorV1::from_bytes(&other).is_none());
    other = bytes.clone();
    other[1] = AccountLayoutVersion::V4 as u8;
    assert!(ReplayAnchorV1::from_bytes(&other).is_none());
    let mut mutable = bytes.clone();
    ReplayAnchorV1::from_bytes_mut(&mut mutable)
        .unwrap()
        .anchor_count = 99;
    assert_eq!(
        ReplayAnchorV1::from_bytes(&mutable).unwrap().anchor_count,
        99
    );
}

#[test]
fn replay_errors_follow_the_selection_error() {
    assert_eq!(
        error::AuctionError::try_from_code(80).unwrap(),
        error::AuctionError::InvalidVerifierSelectionAccounts
    );
    assert_eq!(error::AuctionError::InvalidReplayPolicy.code(), 81);
    assert_eq!(error::AuctionError::InvalidReplayAnchor.code(), 82);
    assert_eq!(error::AuctionError::ReplayAnchorOutOfOrder.code(), 83);
    assert_eq!(
        error::AuctionError::try_from_code(83),
        Ok(error::AuctionError::ReplayAnchorOutOfOrder)
    );
}
