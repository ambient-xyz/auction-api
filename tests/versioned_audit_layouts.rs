use ambient_auction_api::{
    AccountLayoutVersion, BundleEscrowV2, BundleVerifierPageV2, InstructionBytes,
    OpenBundleEscrowV2Args, OpenBundleEscrowV4Args,
};
use bytemuck::Zeroable;

#[test]
fn historical_account_bytes_and_open_payload_remain_unchanged() {
    // Fixed wire offsets, independent of the Rust layouts and their size constants.
    for (version, escrow_len, page_len) in [(1, 504, 816), (2, 568, 880)] {
        let mut escrow = vec![0; escrow_len];
        escrow[0] = 8;
        escrow[1] = version;
        escrow[88..92].copy_from_slice(&17u32.to_le_bytes());
        let decoded = BundleEscrowV2::from_bytes(&escrow).unwrap();
        assert_eq!(decoded.bundle_version, 17);
        assert!(decoded.lifecycle().is_none());
        assert!(decoded.pricing().is_none());
        let mut encoded = vec![0; escrow_len];
        assert!(decoded.write_bytes_with_layout(&mut encoded, decoded.layout().version));
        assert_eq!(encoded, escrow);

        let mut page = vec![0; page_len];
        page[0] = 9;
        page[1] = version;
        page[40..42].copy_from_slice(&2u16.to_le_bytes());
        let decoded = BundleVerifierPageV2::from_bytes(&page).unwrap();
        assert_eq!(decoded.page_index, 2);
        assert!(decoded.lifecycle().is_none());
        assert!(decoded.pricing().is_none());
        let mut encoded = vec![0; page_len];
        assert!(decoded.write_bytes_with_layout(&mut encoded, decoded.layout().version));
        assert_eq!(encoded, page);
    }
    let mut old_open = vec![0; 137];
    old_open[0] = 12;
    old_open[1..5].copy_from_slice(&17u32.to_le_bytes());
    let decoded = OpenBundleEscrowV2Args::try_from(&old_open[1..]).unwrap();
    assert_eq!(decoded.bundle_version, 17);
    assert_eq!(decoded.to_bytes(), old_open);
    assert!(OpenBundleEscrowV4Args::try_from(&old_open[1..]).is_err());
}

#[test]
fn new_metadata_round_trips_and_cannot_be_read_as_an_old_layout() {
    let mut escrow = vec![0; BundleEscrowV2::LEN_V4];
    assert!(
        BundleEscrowV2::default().write_bytes_with_layout(&mut escrow, AccountLayoutVersion::V4)
    );
    {
        let mut state = BundleEscrowV2::from_bytes_mut(&mut escrow).unwrap();
        state.reserved_v2_mut().unwrap().verifier_quorum = 2;
        state.lifecycle_mut().unwrap().expected_page_count = 3;
        state.lifecycle_mut().unwrap().allocated_page_bitmap = 0b101001;
        state.lifecycle_mut().unwrap().small_credit_mint = [9; 32].into();
        state.lifecycle_mut().unwrap().small_credit_amount = 1234;
    }
    let state = BundleEscrowV2::from_bytes(&escrow).unwrap();
    assert_eq!(state.reserved_v2().unwrap().verifier_quorum, 2);
    assert_eq!(state.lifecycle().unwrap().expected_page_count, 3);
    assert_eq!(state.lifecycle().unwrap().allocated_page_bitmap, 0b101001);
    assert_eq!(state.lifecycle().unwrap().small_credit_mint, [9; 32]);
    assert_eq!(state.lifecycle().unwrap().small_credit_amount, 1234);
    for version in [1, 2, 3, 5, 6, 255] {
        escrow[1] = version;
        assert!(BundleEscrowV2::from_bytes(&escrow).is_none());
    }
    let mut page = vec![0; BundleVerifierPageV2::LEN_V4];
    assert!(BundleVerifierPageV2::default()
        .write_bytes_with_layout(&mut page, AccountLayoutVersion::V4));
    {
        let mut state = BundleVerifierPageV2::from_bytes_mut(&mut page).unwrap();
        let metadata = state.lifecycle_mut().unwrap();
        metadata.funder = [7; 32].into();
        metadata.settlement_deadline_slot = 123;
    }
    let state = BundleVerifierPageV2::from_bytes(&page).unwrap();
    assert_eq!(state.lifecycle().unwrap().funder, [7; 32]);
    assert_eq!(state.lifecycle().unwrap().settlement_deadline_slot, 123);
    for version in [1, 2, 3, 5, 6, 255] {
        page[1] = version;
        assert!(BundleVerifierPageV2::from_bytes(&page).is_none());
    }
    let mut args = OpenBundleEscrowV4Args::zeroed();
    args.expected_page_count = 3;
    let bytes = args.to_bytes();
    assert_eq!(bytes[0], 26);
    assert_eq!(OpenBundleEscrowV4Args::try_from(&bytes[1..]).unwrap(), args);
    assert!(OpenBundleEscrowV2Args::try_from(&bytes[1..]).is_err());
}

#[test]
fn evidence_hash_excludes_rent_and_pricing_metadata() {
    use ambient_auction_api::bundle_verifier_page_hash_bytes;
    for version in [
        AccountLayoutVersion::V1,
        AccountLayoutVersion::V2,
        AccountLayoutVersion::V4,
    ] {
        let mut bytes = vec![0; BundleVerifierPageV2::account_len(version)];
        assert!(BundleVerifierPageV2::default().write_bytes_with_layout(&mut bytes, version));
        let original = bundle_verifier_page_hash_bytes(&bytes).unwrap().to_vec();
        if version == AccountLayoutVersion::V4 {
            let mut page = BundleVerifierPageV2::from_bytes_mut(&mut bytes).unwrap();
            page.lifecycle_mut().unwrap().funder = [7; 32].into();
            page.lifecycle_mut().unwrap().settlement_deadline_slot = 123;
            page.pricing_mut().unwrap().pricing_entry_count = 1;
            page.pricing_mut().unwrap().pricing_entries[0].price_per_output_token = 9;
            drop(page);
            assert_eq!(bundle_verifier_page_hash_bytes(&bytes).unwrap(), original);
            for index in 0..6 {
                BundleVerifierPageV2::from_bytes_mut(&mut bytes)
                    .unwrap()
                    .lifecycle_mut()
                    .unwrap()
                    .input_tokens[index] = 42;
                assert_ne!(bundle_verifier_page_hash_bytes(&bytes).unwrap(), original);
                bytes[816 + index * 8..824 + index * 8].fill(0);
                assert_eq!(bundle_verifier_page_hash_bytes(&bytes).unwrap(), original);
            }
            bytes[48] = 1;
            assert_ne!(bundle_verifier_page_hash_bytes(&bytes).unwrap(), original);
        } else {
            assert_eq!(original, bytes);
        }
    }
}

#[test]
fn v4_readers_reject_retired_formats_and_malformed_lengths() {
    let mut escrow = vec![0; 656];
    let mut page = vec![0; 1_200];
    assert!(
        BundleEscrowV2::default().write_bytes_with_layout(&mut escrow, AccountLayoutVersion::V4)
    );
    assert!(BundleVerifierPageV2::default()
        .write_bytes_with_layout(&mut page, AccountLayoutVersion::V4));
    assert!(BundleEscrowV2::from_bytes(&escrow).is_some());
    assert!(BundleEscrowV2::from_bytes_mut(&mut escrow).is_some());
    assert!(BundleVerifierPageV2::from_bytes(&page).is_some());
    assert!(BundleVerifierPageV2::from_bytes_mut(&mut page).is_some());

    for other_version in [0, 1, 2, 3, 5, 6, 255] {
        escrow[1] = other_version;
        page[1] = other_version;
        assert!(BundleEscrowV2::from_bytes(&escrow).is_none());
        assert!(BundleEscrowV2::from_bytes_mut(&mut escrow).is_none());
        assert!(BundleVerifierPageV2::from_bytes(&page).is_none());
        assert!(BundleVerifierPageV2::from_bytes_mut(&mut page).is_none());
    }
    escrow[1] = 4;
    page[1] = 4;

    for length in [0, 616, 655, 657] {
        let mut malformed = escrow.clone();
        malformed.resize(length, 0);
        assert!(BundleEscrowV2::from_bytes(&malformed).is_none());
        assert!(BundleEscrowV2::from_bytes_mut(&mut malformed).is_none());
    }
    for length in [0, 904, 1_199, 1_201] {
        let mut malformed = page.clone();
        malformed.resize(length, 0);
        assert!(BundleVerifierPageV2::from_bytes(&malformed).is_none());
        assert!(BundleVerifierPageV2::from_bytes_mut(&mut malformed).is_none());
    }
}

#[test]
fn small_credit_claim_has_a_fixed_recipient_account_order_and_empty_payload() {
    use ambient_auction_api::{
        ClaimSmallCreditsV4Accounts, ClaimSmallCreditsV4Args, InstructionAccounts,
    };
    let keys = [1, 2, 3, 4, 5];
    let accounts = ClaimSmallCreditsV4Accounts::try_from(keys.as_slice()).unwrap();
    assert_eq!(*accounts.bundle_escrow, 1);
    assert_eq!(*accounts.config_policy, 2);
    assert_eq!(*accounts.mint, 3);
    assert_eq!(*accounts.token_account, 4);
    assert_eq!(*accounts.token_program, 5);
    assert_eq!(accounts.iter().copied().collect::<Vec<_>>(), keys);
    assert_eq!(ClaimSmallCreditsV4Args {}.to_bytes(), [29]);
    assert!(ClaimSmallCreditsV4Args::try_from(&[1][..]).is_err());
}
