use ambient_auction_api::{
    AccountLayoutVersion, BundleEscrowV2, BundleVerifierPageV2, InstructionBytes,
    OpenBundleEscrowV2Args, OpenBundleEscrowV5Args,
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
        assert!(decoded.v5().is_none());
        let mut encoded = vec![0; escrow_len];
        assert!(decoded.write_bytes_with_layout(&mut encoded, decoded.layout().version));
        assert_eq!(encoded, escrow);

        let mut page = vec![0; page_len];
        page[0] = 9;
        page[1] = version;
        page[40..42].copy_from_slice(&2u16.to_le_bytes());
        let decoded = BundleVerifierPageV2::from_bytes(&page).unwrap();
        assert_eq!(decoded.page_index, 2);
        assert!(decoded.v5().is_none());
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
    assert!(OpenBundleEscrowV5Args::try_from(&old_open[1..]).is_err());
}

#[test]
fn new_metadata_round_trips_and_cannot_be_read_as_an_old_layout() {
    let mut escrow = vec![0; BundleEscrowV2::LEN_V5];
    assert!(
        BundleEscrowV2::default().write_bytes_with_layout(&mut escrow, AccountLayoutVersion::V5)
    );
    {
        let mut state = BundleEscrowV2::from_bytes_mut(&mut escrow).unwrap();
        state.reserved_v2_mut().unwrap().verifier_quorum = 2;
        state.v5_mut().unwrap().expected_page_count = 3;
        state.v5_mut().unwrap().allocated_page_bitmap = 0b101001;
    }
    let state = BundleEscrowV2::from_bytes(&escrow).unwrap();
    assert_eq!(state.reserved_v2().unwrap().verifier_quorum, 2);
    assert_eq!(state.v5().unwrap().expected_page_count, 3);
    assert_eq!(state.v5().unwrap().allocated_page_bitmap, 0b101001);
    for version in [1, 2, 3, 4, 255] {
        escrow[1] = version;
        assert!(BundleEscrowV2::from_bytes(&escrow).is_none());
    }
    let mut page = vec![0; BundleVerifierPageV2::LEN_V5];
    assert!(BundleVerifierPageV2::default()
        .write_bytes_with_layout(&mut page, AccountLayoutVersion::V5));
    {
        let mut state = BundleVerifierPageV2::from_bytes_mut(&mut page).unwrap();
        let metadata = state.v5_mut().unwrap();
        metadata.funder = [7; 32].into();
        metadata.settlement_deadline_slot = 123;
    }
    let state = BundleVerifierPageV2::from_bytes(&page).unwrap();
    assert_eq!(state.v5().unwrap().funder, [7; 32]);
    assert_eq!(state.v5().unwrap().settlement_deadline_slot, 123);
    for version in [1, 2, 3, 4, 255] {
        page[1] = version;
        assert!(BundleVerifierPageV2::from_bytes(&page).is_none());
    }
    let mut args = OpenBundleEscrowV5Args::zeroed();
    args.expected_page_count = 3;
    let bytes = args.to_bytes();
    assert_eq!(bytes[0], 26);
    assert_eq!(OpenBundleEscrowV5Args::try_from(&bytes[1..]).unwrap(), args);
    assert!(OpenBundleEscrowV2Args::try_from(&bytes[1..]).is_err());
}

#[test]
fn evidence_hash_excludes_only_new_rent_metadata() {
    use ambient_auction_api::bundle_verifier_page_hash_bytes;
    for version in [
        AccountLayoutVersion::V1,
        AccountLayoutVersion::V2,
        AccountLayoutVersion::V5,
    ] {
        let mut bytes = vec![0; BundleVerifierPageV2::account_len(version)];
        assert!(BundleVerifierPageV2::default().write_bytes_with_layout(&mut bytes, version));
        let original = bundle_verifier_page_hash_bytes(&bytes).unwrap().to_vec();
        if version == AccountLayoutVersion::V5 {
            BundleVerifierPageV2::from_bytes_mut(&mut bytes)
                .unwrap()
                .v5_mut()
                .unwrap()
                .funder = [7; 32].into();
            assert_eq!(bundle_verifier_page_hash_bytes(&bytes).unwrap(), original);
            bytes[48] = 1;
            assert_ne!(bundle_verifier_page_hash_bytes(&bytes).unwrap(), original);
        } else {
            assert_eq!(original, bytes);
        }
    }
}
