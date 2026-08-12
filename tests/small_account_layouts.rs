use ambient_auction_api::{
    AccountLayoutVersion, BundleEscrowV2, BundleEscrowV2ReservedData, BundleEscrowV3SmallData,
    BundleVerifierPageReservedData, BundleVerifierPageV2, BundleVerifierPageV3SmallData, Pubkey,
};
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn small_account_layouts_are_exact() {
    assert_eq!(AccountLayoutVersion::V3 as u8, 3);
    assert_eq!(BundleEscrowV2::LEN_V1, 504);
    assert_eq!(BundleEscrowV2::LEN_V2, 568);
    assert_eq!(BundleEscrowV2::LEN_V3, 568);
    assert_eq!(size_of::<BundleEscrowV2ReservedData>(), 64);
    assert_eq!(size_of::<BundleEscrowV3SmallData>(), 64);
    assert_eq!(offset_of!(BundleEscrowV3SmallData, mint), 0);
    assert_eq!(offset_of!(BundleEscrowV3SmallData, _reserved), 32);

    assert_eq!(BundleVerifierPageV2::LEN_V1, 816);
    assert_eq!(BundleVerifierPageV2::LEN_V2, 880);
    assert_eq!(BundleVerifierPageV2::LEN_V3, 880);
    assert_eq!(size_of::<BundleVerifierPageReservedData>(), 64);
    assert_eq!(size_of::<BundleVerifierPageV3SmallData>(), 64);
    assert_eq!(offset_of!(BundleVerifierPageV3SmallData, input_tokens), 0);
    assert_eq!(offset_of!(BundleVerifierPageV3SmallData, _reserved), 48);
}

#[test]
fn v2_reserves_and_v3_small_tails_round_trip() {
    let raw = BundleEscrowV2::default();
    let mut v2 = vec![0; BundleEscrowV2::LEN_V2];
    assert!(raw.write_v2_bytes(&mut v2));
    assert!(BundleEscrowV2::from_bytes(&v2)
        .unwrap()
        .reserved_v2()
        .unwrap()
        .is_zero());
    v2[BundleEscrowV2::LEN_V1] = 1;
    assert!(!BundleEscrowV2::from_bytes(&v2)
        .unwrap()
        .reserved_v2()
        .unwrap()
        .is_zero());

    let mut v3 = vec![0; BundleEscrowV2::LEN_V3];
    assert!(raw.write_v3_bytes(&mut v3));
    *BundleEscrowV2::from_bytes_mut(&mut v3)
        .unwrap()
        .small_v3_mut()
        .unwrap() = BundleEscrowV3SmallData {
        mint: Pubkey::from([7; 32]),
        _reserved: [0; 32],
    };
    assert_eq!(
        BundleEscrowV2::from_bytes(&v3)
            .unwrap()
            .small_v3()
            .unwrap()
            .mint,
        Pubkey::from([7; 32])
    );

    let page_raw = BundleVerifierPageV2::default();
    let mut page_v2 = vec![0; BundleVerifierPageV2::LEN_V2];
    assert!(page_raw.write_v2_bytes(&mut page_v2));
    assert!(BundleVerifierPageV2::from_bytes(&page_v2)
        .unwrap()
        .reserved_v2()
        .unwrap()
        .is_zero());

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
