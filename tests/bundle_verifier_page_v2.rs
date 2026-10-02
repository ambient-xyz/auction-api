use ambient_auction_api::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, BundleJobPricingV6,
    BundleVerifierPageV2, BundleVerifierPageV2Entry, BundleVerifierPageV5Data,
    BundleVerifierPageV6Data, Pubkey, VerificationVerdictV2, MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES,
};
use bytemuck::Zeroable;
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn bundle_verifier_page_v2_round_trips_through_bytes() {
    let mut page = BundleVerifierPageV2::zeroed();
    page.write_entries(
        [7; 32].into(),
        2,
        1,
        [BundleVerifierPageV2Entry {
            job_id: Pubkey::from([8; 32]),
            posted_output_tokens: 34,
            accepted_output_tokens: 21,
            assigned_verifiers_token_ranges: [0, 8, 6, 17, 15, 34],
            verifier_reward_tokens: [5, 8, 13],
            verdict: VerificationVerdictV2::Verified,
            verifier_claimed_bitmap: 0,
            _reserved: [0; 6],
        }; MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
    );

    let mut bytes = vec![0u8; BundleVerifierPageV2::LEN];
    assert!(page.write_v1_bytes(&mut bytes));

    let parsed = BundleVerifierPageV2::from_bytes(&bytes).unwrap();
    assert_eq!(
        parsed.header(),
        &AccountHeaderV1::new(AccountDiscriminator::BundleVerifierPageV2)
    );
    assert_eq!(parsed.bundle_escrow, Pubkey::from([7; 32]));
    assert_eq!(parsed.page_index, 2);
    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.entries[0].posted_output_tokens, 34);
    assert_eq!(parsed.entries[0].accepted_output_tokens, 21);
    assert_eq!(
        parsed.entries[0].assigned_verifiers_token_ranges,
        [0, 8, 6, 17, 15, 34]
    );
    assert_eq!(parsed.entries[0].verifier_reward_tokens, [5, 8, 13]);
}

#[test]
fn bundle_verifier_page_v2_v2_round_trips_through_bytes() {
    let mut page = BundleVerifierPageV2::zeroed();
    page.write_entries(
        [3; 32].into(),
        1,
        0,
        [BundleVerifierPageV2Entry::default(); MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
    );

    let mut bytes = vec![0u8; BundleVerifierPageV2::LEN_V2];
    assert!(page.write_v2_bytes(&mut bytes));

    let parsed = BundleVerifierPageV2::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.layout().version, AccountLayoutVersion::V2);
    assert_eq!(parsed.bundle_escrow, Pubkey::from([3; 32]));
    assert_eq!(parsed.page_index, 1);
    assert_eq!(parsed.entry_count, 0);
}

#[test]
fn bundle_verifier_page_v2_rejects_wrong_lengths() {
    let mut bytes = vec![0u8; BundleVerifierPageV2::LEN - 1];
    assert!(BundleVerifierPageV2::from_bytes(&bytes).is_none());
    bytes.push(0);
    assert!(BundleVerifierPageV2::from_bytes(&bytes).is_none());
}

#[test]
fn bundle_verifier_page_v2_entry_layout_stays_stable() {
    assert_eq!(size_of::<BundleVerifierPageV2Entry>(), 128);
    assert_eq!(BundleVerifierPageV2::PAYLOAD_LEN, 808);
}

#[test]
fn bundle_verifier_page_v2_entry_capacity_is_a_protocol_cap() {
    assert_eq!(MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES, 6);
}

#[test]
fn bundle_verifier_page_v6_layout_is_append_only_and_stable() {
    assert_eq!(size_of::<BundleJobPricingV6>(), 48);
    assert_eq!(offset_of!(BundleJobPricingV6, job_id), 0);
    assert_eq!(offset_of!(BundleJobPricingV6, max_output_tokens), 32);
    assert_eq!(offset_of!(BundleJobPricingV6, price_per_output_token), 40);

    assert_eq!(size_of::<BundleVerifierPageV5Data>(), 88);
    assert_eq!(size_of::<BundleVerifierPageV6Data>(), 296);
    assert_eq!(offset_of!(BundleVerifierPageV6Data, pricing_entry_count), 0);
    assert_eq!(offset_of!(BundleVerifierPageV6Data, pricing_entries), 8);

    assert_eq!(BundleVerifierPageV2::LEN_V5, 904);
    assert_eq!(BundleVerifierPageV2::LEN_V6, 1_200);
    assert_eq!(
        BundleVerifierPageV2::LEN_V6,
        BundleVerifierPageV2::LEN_V5 + size_of::<BundleVerifierPageV6Data>()
    );
    assert_eq!(
        BundleVerifierPageV2::account_len(AccountLayoutVersion::V6),
        BundleVerifierPageV2::LEN_V6
    );
}

#[test]
fn bundle_verifier_page_v6_round_trips_v5_and_pricing_tails() {
    let mut page = BundleVerifierPageV2::zeroed();
    assert!(page.write_entries(
        [3; 32].into(),
        1,
        2,
        [BundleVerifierPageV2Entry::default(); MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
    ));
    let mut bytes = vec![0xFF; BundleVerifierPageV2::LEN_V6];
    assert!(page.write_bytes_with_layout(&mut bytes, AccountLayoutVersion::V6));

    // The writer must initialize both the inherited V5 prefix and new V6 tail.
    {
        let state = BundleVerifierPageV2::from_bytes(&bytes).unwrap();
        assert_eq!(state.layout().version, AccountLayoutVersion::V6);
        assert_eq!(state.bundle_escrow, Pubkey::from([3; 32]));
        assert_eq!(state.page_index, 1);
        assert_eq!(state.entry_count, 2);
        assert_eq!(state.v5().unwrap(), &BundleVerifierPageV5Data::default());
        assert_eq!(state.v6().unwrap(), &BundleVerifierPageV6Data::default());
    }

    let first_price = BundleJobPricingV6 {
        job_id: [7; 32].into(),
        max_output_tokens: 100,
        price_per_output_token: 11,
    };
    let second_price = BundleJobPricingV6 {
        job_id: [8; 32].into(),
        max_output_tokens: 200,
        price_per_output_token: 13,
    };
    {
        let mut state = BundleVerifierPageV2::from_bytes_mut(&mut bytes).unwrap();
        let rent = state.v5_mut().unwrap();
        rent.input_tokens = [1, 2, 3, 4, 5, 6];
        rent.funder = [6; 32].into();
        rent.settlement_deadline_slot = 123;

        let pricing = state.v6_mut().unwrap();
        pricing.pricing_entry_count = 2;
        pricing.pricing_entries[0] = first_price;
        pricing.pricing_entries[1] = second_price;
    }

    let state = BundleVerifierPageV2::from_bytes(&bytes).unwrap();
    assert_eq!(state.v5().unwrap().input_tokens, [1, 2, 3, 4, 5, 6]);
    assert_eq!(state.v5().unwrap().funder, [6; 32]);
    assert_eq!(state.v5().unwrap().settlement_deadline_slot, 123);
    assert_eq!(state.v6().unwrap().pricing_entry_count, 2);
    assert_eq!(state.v6().unwrap().pricing_entries[0], first_price);
    assert_eq!(state.v6().unwrap().pricing_entries[1], second_price);
    assert_eq!(&bytes[816..824], &1_u64.to_le_bytes());
    assert_eq!(&bytes[864..896], &[6; 32]);
    assert_eq!(bytes[904], 2);
    assert_eq!(
        state.v6().unwrap().pricing_entries[2..],
        [BundleJobPricingV6::default(); MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES - 2]
    );

    // This catches a parser that accidentally treats the V6 tail as the V5 prefix.
    assert!(BundleVerifierPageV2::from_bytes(&bytes[..BundleVerifierPageV2::LEN_V5]).is_none());
    assert!(
        BundleVerifierPageV2::from_bytes_mut(&mut bytes[..BundleVerifierPageV2::LEN_V5]).is_none()
    );
}
