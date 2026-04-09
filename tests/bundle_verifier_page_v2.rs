use ambient_auction_api::{
    AccountDiscriminator, AccountHeaderV1, BundleVerifierPageV2, BundleVerifierPageV2Entry, Pubkey,
    VerificationVerdictV2, BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES,
};
use bytemuck::Zeroable;
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
        }; BUNDLE_VERIFIER_PAGE_V2_MAX_ENTRIES],
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
fn bundle_verifier_page_v2_rejects_wrong_lengths() {
    let mut bytes = vec![0u8; BundleVerifierPageV2::LEN - 1];
    assert!(BundleVerifierPageV2::from_bytes(&bytes).is_none());
    bytes.push(0);
    assert!(BundleVerifierPageV2::from_bytes(&bytes).is_none());
}

#[test]
fn bundle_verifier_page_v2_entry_layout_stays_stable() {
    assert_eq!(size_of::<BundleVerifierPageV2Entry>(), 128);
    assert_eq!(BundleVerifierPageV2::PAYLOAD_LEN, 1064);
}
