use ambient_auction_api::{AccountLayoutVersion, ConfigPolicyV2, RequestTier};
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn config_policy_v2_default_credit_cap_is_production_bounded() {
    let policy = ConfigPolicyV2::default();

    assert_eq!(policy.max_auction_credits_per_update, 1_000_000_000);
}

#[test]
fn config_policy_v2_default_windows_are_production_stage_windows() {
    let policy = ConfigPolicyV2::production_default();

    for tier in RequestTier::ALL {
        let tier_config = policy.tier_config(tier);
        assert_eq!(tier_config.settlement_window_slots, 32);
        assert_eq!(tier_config.result_window_slots, 32);
        assert_eq!(tier_config.verification_window_slots, 32);
        assert_eq!(tier_config.claim_window_slots, 32);
    }
}

#[test]
fn config_policy_v2_layout_size_stays_stable() {
    assert_eq!(ConfigPolicyV2::LEN, 1_568);
    assert_eq!(size_of::<ConfigPolicyV2>(), ConfigPolicyV2::LEN);
    assert_eq!(std::mem::align_of::<ConfigPolicyV2>(), 8);
    assert_eq!(offset_of!(ConfigPolicyV2, small_credit_mint), 1_320);
    assert_eq!(offset_of!(ConfigPolicyV2, small_credit_enabled), 1_352);
    assert_eq!(
        offset_of!(ConfigPolicyV2, small_credit_slash_authority),
        1_384
    );
    assert_eq!(
        offset_of!(ConfigPolicyV2, small_credit_slash_sequence),
        1_416
    );
    assert_eq!(offset_of!(ConfigPolicyV2, reserved_words), 1_448);
    assert_eq!(offset_of!(ConfigPolicyV2, v2_account_layout_version), 1_544);
    assert_eq!(
        offset_of!(ConfigPolicyV2, max_auction_credits_per_update),
        24
    );
    assert_eq!(
        offset_of!(ConfigPolicyV2, missed_verification_dispute_window_slots),
        1_288
    );
}

#[test]
fn config_policy_v2_round_trips_through_bytes() {
    let mut policy = ConfigPolicyV2 {
        max_auction_credits_per_update: 42,
        missed_verification_dispute_window_slots: 5,
        dispute_verification_window_slots: 7,
        paid_verification_dispute_window_slots: 11,
        paid_verification_dispute_bond_lamports: 13,
        ..ConfigPolicyV2::default()
    };

    assert_eq!(policy.paid_dispute_bond_bps(), 0);
    policy.set_paid_dispute_bond_bps(500);
    let decoded = *bytemuck::from_bytes::<ConfigPolicyV2>(bytemuck::bytes_of(&policy));
    assert_eq!(decoded.paid_dispute_bond_bps(), 500);
    assert_eq!(decoded, policy);
    assert_eq!(
        decoded.configured_v2_account_layout_version(),
        Ok(AccountLayoutVersion::V4)
    );
}

#[test]
fn config_policy_v2_rejects_unsupported_versions_and_keeps_the_v4_default() {
    let mut policy = ConfigPolicyV2::default();
    assert_eq!(
        policy.configured_v2_account_layout_version(),
        Ok(AccountLayoutVersion::V4)
    );

    assert_eq!(bytemuck::bytes_of(&policy)[1_544], 4);
    for version in [5, 6] {
        policy.v2_account_layout_version = version;
        assert_eq!(policy.configured_v2_account_layout_version(), Err(version));
    }

    policy.v2_account_layout_version = 3;
    assert_eq!(policy.configured_v2_account_layout_version(), Err(3));
}

#[test]
fn paid_dispute_bond_uses_the_saved_rate_and_rounds_up() {
    let mut escrow = ambient_auction_api::BundleEscrowV2 {
        winner_payout_lamports: u64::MAX,
        ..ambient_auction_api::BundleEscrowV2::default()
    };
    assert_eq!(escrow.paid_dispute_bond_lamports(19), Some(19));
    escrow.set_paid_dispute_bond_bps(10_000);
    assert_eq!(escrow.paid_dispute_bond_lamports(19), Some(u64::MAX));
    escrow.set_paid_dispute_bond_bps(100);
    escrow.winner_payout_lamports = 1_901;
    assert_eq!(escrow.paid_dispute_bond_lamports(19), Some(20));
    escrow.winner_payout_lamports = 0;
    assert_eq!(escrow.paid_dispute_bond_lamports(19), Some(19));
}
