use ambient_auction_api::{
    AccountLayoutVersion, BundleEscrowV2, BundleEscrowV3SmallData, BundleVerifierPageV2,
    BundleVerifierPageV3SmallData, Pubkey,
};
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn small_account_layouts_are_exact() {
    assert_eq!(AccountLayoutVersion::V3 as u8, 3);
    assert_eq!(BundleEscrowV2::LEN_V1, 504);
    assert_eq!(BundleEscrowV2::LEN_V2, 568);
    assert_eq!(BundleEscrowV2::LEN_V3, 536);
    assert_eq!(size_of::<BundleEscrowV3SmallData>(), 32);
    assert_eq!(offset_of!(BundleEscrowV3SmallData, mint), 0);

    assert_eq!(BundleVerifierPageV2::LEN_V1, 816);
    assert_eq!(BundleVerifierPageV2::LEN_V2, 880);
    assert_eq!(BundleVerifierPageV2::LEN_V3, 864);
    assert_eq!(size_of::<BundleVerifierPageV3SmallData>(), 48);
    assert_eq!(offset_of!(BundleVerifierPageV3SmallData, input_tokens), 0);
}

#[test]
fn v2_policy_terms_and_v3_small_data_round_trip() {
    let raw = BundleEscrowV2::default();
    let mut v2 = vec![0; BundleEscrowV2::LEN_V2];
    assert!(raw.write_v2_bytes(&mut v2));
    let mut decoded = BundleEscrowV2::from_bytes_mut(&mut v2).unwrap();
    let terms = decoded.reserved_v2_mut().unwrap();
    terms.paid_verification_dispute_bond_lamports = 23;
    terms.max_auction_credits_per_update = 10_000;
    terms.verifier_count = 3;
    terms.verifier_quorum = 2;
    let decoded = BundleEscrowV2::from_bytes(&v2).unwrap();
    assert_eq!(decoded.reserved_v2().unwrap().verifier_count, 3);
    assert_eq!(
        decoded
            .reserved_v2()
            .unwrap()
            .paid_verification_dispute_bond_lamports,
        23
    );
    assert!(decoded.small_v3().is_none());

    let mut v3 = vec![0; BundleEscrowV2::LEN_V3];
    assert!(raw.write_v3_bytes(&mut v3));
    BundleEscrowV2::from_bytes_mut(&mut v3)
        .unwrap()
        .small_v3_mut()
        .unwrap()
        .mint = Pubkey::from([7; 32]);
    assert_eq!(
        BundleEscrowV2::from_bytes(&v3)
            .unwrap()
            .small_v3()
            .unwrap()
            .mint,
        Pubkey::from([7; 32])
    );

    assert!(BundleEscrowV2::from_bytes(&v3)
        .unwrap()
        .reserved_v2()
        .is_none());
    assert!(BundleEscrowV2::from_bytes_mut(&mut v3)
        .unwrap()
        .reserved_v2_mut()
        .is_none());

    let page_raw = BundleVerifierPageV2::default();
    let mut page_v2 = vec![0; BundleVerifierPageV2::LEN_V2];
    assert!(page_raw.write_v2_bytes(&mut page_v2));
    assert!(BundleVerifierPageV2::from_bytes(&page_v2)
        .unwrap()
        .has_canonical_v2_tail());
    page_v2[BundleVerifierPageV2::LEN_V1] = 1;
    assert!(!BundleVerifierPageV2::from_bytes(&page_v2)
        .unwrap()
        .has_canonical_v2_tail());

    let mut page_v3 = vec![0; BundleVerifierPageV2::LEN_V3];
    assert!(page_raw.write_v3_bytes(&mut page_v3));
    BundleVerifierPageV2::from_bytes_mut(&mut page_v3)
        .unwrap()
        .small_v3_mut()
        .unwrap()
        .input_tokens = [1, 2, 3, 4, 5, 6];
    assert_eq!(
        BundleVerifierPageV2::from_bytes(&page_v3)
            .unwrap()
            .small_v3()
            .unwrap()
            .input_tokens,
        [1, 2, 3, 4, 5, 6]
    );
}
