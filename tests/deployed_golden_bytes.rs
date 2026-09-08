use ambient_auction_api::{
    BundleEscrowV2, BundleEscrowV2Status, BundleVerifierPageV2, BundleVerifierPageV2Entry,
    ConfigPolicyV2, ConfigPolicyV2Flag, ConfigPolicyV2Flags, Pubkey, RequestTier,
    VerificationVerdictV2, MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES,
};

fn key(byte: u8) -> Pubkey {
    [byte; 32].into()
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[test]
fn deployed_v2_account_bytes_remain_stable() {
    let escrow = BundleEscrowV2 {
        status: BundleEscrowV2Status::ResultPosted,
        reward_tier: u64::from(RequestTier::Pro),
        coordinator: key(1),
        requester_refund_recipient: key(2),
        bundle_version: 0x0304_0506,
        _reserved0: [7, 8, 9, 10],
        bundle_hash: [11; 32],
        total_input_tokens: 0x1112_1314_1516_1718,
        escrow_lamports: 0x2122_2324_2526_2728,
        winner_vote_account: key(12),
        selected_verifiers: [key(13), key(14), key(15)],
        result_hash: [16; 32],
        posted_output_tokens: 0x3132_3334_3536_3738,
        claim_deadline_slot: 0x4142_4344_4546_4748,
        verifier_page_count: 3,
        _reserved1: [17, 18, 19, 20],
        verifier_reward_remaining: [21, 22, 23],
        ..Default::default()
    };
    let mut escrow_bytes = vec![0; BundleEscrowV2::LEN_V2];
    assert!(escrow.write_v2_bytes(&mut escrow_bytes));
    let mut escrow_v1_bytes = vec![0; BundleEscrowV2::LEN_V1];
    assert!(escrow.write_v1_bytes(&mut escrow_v1_bytes));
    let mut escrow_v2_prefix = escrow_bytes[..BundleEscrowV2::LEN_V1].to_vec();
    escrow_v2_prefix[1] = 1;
    assert_eq!(escrow_v1_bytes, escrow_v2_prefix);

    let entry = BundleVerifierPageV2Entry {
        job_id: key(31),
        posted_output_tokens: 32,
        accepted_output_tokens: 33,
        assigned_verifiers_token_ranges: [34, 35, 36, 37, 38, 39],
        verifier_reward_tokens: [40, 41, 42],
        verdict: VerificationVerdictV2::Verified,
        verifier_claimed_bitmap: 0b101,
        _reserved: [43; 6],
    };
    let mut page = BundleVerifierPageV2::default();
    assert!(page.write_entries(
        key(30),
        0x1234,
        6,
        [entry; MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES],
    ));
    let mut page_bytes = vec![0; BundleVerifierPageV2::LEN_V2];
    assert!(page.write_v2_bytes(&mut page_bytes));
    let mut page_v1_bytes = vec![0; BundleVerifierPageV2::LEN_V1];
    assert!(page.write_v1_bytes(&mut page_v1_bytes));
    let mut page_v2_prefix = page_bytes[..BundleVerifierPageV2::LEN_V1].to_vec();
    page_v2_prefix[1] = 1;
    assert_eq!(page_v1_bytes, page_v2_prefix);

    let mut admin_authorities = [Pubkey::default(); 8];
    admin_authorities[0] = key(53);
    admin_authorities[7] = key(54);
    let mut service_authorities = [Pubkey::default(); 16];
    service_authorities[0] = key(55);
    service_authorities[15] = key(56);
    let mut reserved_words = [[0; 32]; 7];
    reserved_words[6] = [59; 32];
    let policy = ConfigPolicyV2 {
        bump: 51,
        minimum_bundle_auction_pairs: 52,
        policy_flags: ConfigPolicyV2Flags::from_flag(
            ConfigPolicyV2Flag::AllowServicePageBackedFinalizeBypass,
        ),
        admin_authorities,
        service_authorities,
        _reserved1: [57; 6],
        missed_verification_dispute_window_slots: u64::from_le_bytes([58; 8]),
        dispute_verification_window_slots: u64::from_le_bytes([58; 8]),
        paid_verification_dispute_window_slots: u64::from_le_bytes([58; 8]),
        paid_verification_dispute_bond_lamports: u64::from_le_bytes([58; 8]),
        reserved_words,
        _reserved2: [60; 7],
        reserved_tail: [61; 16],
        ..ConfigPolicyV2::production_default()
    };

    assert_eq!(fnv1a64(&escrow_bytes), 0xa57b_1b7d_f638_73f2);
    assert_eq!(fnv1a64(&page_bytes), 0xe2f4_3c61_83a5_73c2);
    assert_eq!(fnv1a64(bytemuck::bytes_of(&policy)), 0xa06e_cf37_5b6f_d6fb);
}
