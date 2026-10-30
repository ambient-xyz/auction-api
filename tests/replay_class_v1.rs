use ambient_auction_api::*;
use memoffset::offset_of;

fn key(byte: u8) -> Pubkey {
    Pubkey::from([byte; 32])
}

#[test]
fn replay_class_layout() {
    assert_eq!(offset_of!(RawReplayClassV1Data, class_id), 0);
    assert_eq!(offset_of!(RawReplayClassV1Data, member_count), 32);
    assert_eq!(offset_of!(RawReplayClassV1Data, bump), 34);
    assert_eq!(offset_of!(RawReplayClassV1Data, members), 40);
    assert_eq!(ReplayClassV1::LEN, 8 + 40 + 32 * REPLAY_CLASS_V1_CAPACITY);
    assert_eq!(AccountDiscriminator::ReplayClassV1 as u8, 12);
    for (instruction, tag) in [
        (AuctionInstruction::JoinReplayClassV1, 37),
        (AuctionInstruction::LeaveReplayClassV1, 38),
        (AuctionInstruction::RemoveReplayClassMemberV1, 39),
        (AuctionInstruction::SelectReplayVerifiersV1, 40),
    ] {
        assert_eq!(instruction as u8, tag);
    }
    assert_eq!(std::mem::size_of::<JoinReplayClassV1Args>(), 32);
    assert_eq!(std::mem::size_of::<RemoveReplayClassMemberV1Args>(), 64);
    assert_eq!(std::mem::size_of::<SelectReplayVerifiersV1Args>(), 0);
    for (error, code) in [
        (error::AuctionError::InvalidReplayClass, 84),
        (error::AuctionError::ReplayClassMembership, 85),
        (error::AuctionError::ReplayClassFull, 86),
    ] {
        assert_eq!(error.code(), code);
    }
}

#[test]
fn members_stay_sorted_unique_and_bounded() {
    let mut class = ReplayClassV1 {
        class_id: [5; 32],
        ..ReplayClassV1::default()
    };
    assert!(class.is_canonical());
    for byte in [9, 3, 7, 1] {
        assert!(class.insert(key(byte)));
    }
    assert!(!class.insert(key(7)), "already a member");
    assert!(!class.insert(Pubkey::default()), "the zero key");
    assert_eq!(class.members(), &[key(1), key(3), key(7), key(9)]);
    assert!(class.contains(&key(3)) && !class.contains(&key(4)));
    assert!(class.is_canonical());

    assert!(class.remove(&key(3)));
    assert!(!class.remove(&key(3)));
    assert_eq!(class.members(), &[key(1), key(7), key(9)]);
    assert!(class.is_canonical());

    let mut full = ReplayClassV1::default();
    for byte in 1..=REPLAY_CLASS_V1_CAPACITY as u8 {
        assert!(full.insert(key(byte)));
    }
    assert!(!full.insert(key(200)), "full");
    assert_eq!(usize::from(full.member_count), REPLAY_CLASS_V1_CAPACITY);
    assert!(full.is_canonical());

    let mut broken = class;
    broken.members[0] = key(8);
    assert!(!broken.is_canonical(), "out of order");
    broken = class;
    broken.members[10] = key(1);
    assert!(!broken.is_canonical(), "a key past the count");
    broken = class;
    broken.member_count = REPLAY_CLASS_V1_CAPACITY as u16 + 1;
    assert!(!broken.is_canonical(), "a count past the capacity");
}

#[test]
fn class_accounts_round_trip_and_reject_other_layouts() {
    let mut class = ReplayClassV1 {
        class_id: [5; 32],
        bump: 251,
        ..ReplayClassV1::default()
    };
    class.insert(key(2));
    let mut bytes = vec![0; ReplayClassV1::LEN];
    assert!(class.write_bytes(&mut bytes));
    assert_eq!(*ReplayClassV1::from_bytes(&bytes).unwrap(), class);
    ReplayClassV1::from_bytes_mut(&mut bytes)
        .unwrap()
        .insert(key(1));
    assert_eq!(
        ReplayClassV1::from_bytes(&bytes).unwrap().members(),
        &[key(1), key(2)]
    );
    assert!(ReplayClassV1::from_bytes(&bytes[1..]).is_none());
    let mut other = bytes.clone();
    other[0] = AccountDiscriminator::ReplayAnchorV1 as u8;
    assert!(ReplayClassV1::from_bytes(&other).is_none());
}

#[test]
fn replay_selection_accounts_take_an_optional_dispute_only() {
    let accounts = [1u8, 2, 3, 4, 5];
    let parsed = SelectReplayVerifiersV1Accounts::try_from(&accounts[..]).unwrap();
    assert!(parsed.bundle_verification_dispute.is_none());
    assert_eq!(*parsed.replay_class, 4);
    let with_dispute = [1u8, 2, 3, 4, 5, 6];
    assert_eq!(
        SelectReplayVerifiersV1Accounts::try_from(&with_dispute[..])
            .unwrap()
            .bundle_verification_dispute,
        Some(&6)
    );
    assert!(SelectReplayVerifiersV1Accounts::try_from(&accounts[..4]).is_err());
    assert_eq!(
        SelectReplayVerifiersV1Accounts::try_from(&[1u8, 2, 3, 4, 5, 6, 7][..]).err(),
        Some(error::AuctionError::InvalidVerifierSelectionAccounts)
    );
}
