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
    assert_eq!(
        offset_of!(ConfigPolicyV2, max_auction_credits_per_update),
        24
    );
}

#[test]
fn config_policy_v2_round_trips_through_bytes() {
    let policy = ConfigPolicyV2 {
        max_auction_credits_per_update: 42,
        ..ConfigPolicyV2::default()
    };

    let decoded = *bytemuck::from_bytes::<ConfigPolicyV2>(bytemuck::bytes_of(&policy));
    assert_eq!(decoded, policy);
    assert_eq!(
        decoded.configured_v2_account_layout_version(),
        Ok(AccountLayoutVersion::V2)
    );
}
