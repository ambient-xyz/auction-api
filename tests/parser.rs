#![cfg(feature = "parser")]

use {
    ambient_auction_api::{
        parser::{parse_instruction, ParseInstructionError},
        AnchorReplaySpansV1Args, AuthorizeBundleDisputeEvidenceV4Args, BundleJobPricingV4,
        BundleVerificationDisputeV2Kind, BundleVerifierPageV2Entry, ClaimSmallCreditsV4Args,
        CloseBundleVerifierPageV4Args, CommitAuctionSettlementV3Args,
        CommitSmallAuctionSettlementV4Args, ConfigPolicyV2Flags, ConfigPolicyV2PatchKind,
        DisputeBundleVerificationV2Args, InitConfigPolicySmallV3Args, InitConfigPolicyV2Args,
        InstructionBytes, OpenBundleEscrowV4Args, OpenPricedBundleEscrowV4Args,
        OpenSmallBundleEscrowV4Args, PostBundlePricingArgs, PostBundleResultV2Args,
        PostBundleResultV3Args, ReplayPolicyV1, RequestTierConfigV2, SealBundlePricingArgs,
        SelectBundleVerifiersV2Args, SetConfigPolicyReplayV1Args, SetConfigPolicySmallV3Args,
        SetConfigPolicyV2Args, SlashSmallCreditsArgs, VerificationVerdictV2,
    },
    bytemuck::Zeroable,
    serde_json::{json, Value},
};

const HASH: &str = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=";
const ZERO_HASH: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

fn key(index: u8) -> String {
    bs58::encode([index; 32]).into_string()
}

fn assert_parsed(data: Vec<u8>, minimum_accounts: usize, accounts: &[u8], expected: Value) {
    let accounts: Vec<_> = accounts.iter().map(|index| [*index; 32]).collect();
    assert_eq!(
        serde_json::to_value(parse_instruction(&data, &accounts).unwrap()).unwrap(),
        expected,
    );
    for count in 0..minimum_accounts {
        assert_eq!(
            parse_instruction(&data, &accounts[..count]),
            Err(ParseInstructionError::InstructionKeyMismatch),
            "{} with {count} accounts",
            expected["type"],
        );
    }
    for oversized in [false, true] {
        let mut invalid = data.clone();
        if oversized {
            invalid.push(0);
        } else {
            invalid.pop();
        }
        assert_eq!(
            parse_instruction(&invalid, &accounts),
            Err(ParseInstructionError::InstructionNotParsable),
            "{} with {} data bytes",
            expected["type"],
            invalid.len(),
        );
    }
}

#[test]
fn disputes_and_selection() {
    assert_parsed(
        DisputeBundleVerificationV2Args {
            kind: BundleVerificationDisputeV2Kind::PaidVerdictDispute,
            _reserved: [0; 7],
        }
        .to_bytes(),
        6,
        &[3, 1, 5, 0, 4, 2],
        json!({
            "type": "dispute_bundle_verification_v2", "info": {
                "dispute_payer": key(3), "bundle_escrow": key(1),
                "bundle_verification_dispute": key(5), "bond_refund_recipient": key(0),
                "config_policy": key(4), "system_program": key(2),
                "kind": {"name": "paid_verdict_dispute", "value": 2},
            }
        }),
    );
    for accounts in [&[3, 1, 5][..], &[3, 1, 5, 0][..]] {
        assert_parsed(
            SelectBundleVerifiersV2Args {}.to_bytes(),
            3,
            accounts,
            json!({
                "type": "select_bundle_verifiers_v2", "info": {
                    "bundle_escrow": key(3), "auction_verifiers": key(1), "slot_hashes": key(5),
                    "bundle_verification_dispute": if accounts.len() == 4 { Some(key(0)) } else { None },
                }
            }),
        );
    }
    assert_parsed(
        AuthorizeBundleDisputeEvidenceV4Args {
            verification_hash: [1; 32],
            page_hashes: [[1; 32], [0; 32], [0; 32]],
            quorum_verifier_bitmap: 5,
            _reserved: [0; 7],
        }
        .to_bytes(),
        4,
        &[3, 1, 5, 0],
        json!({
            "type": "authorize_bundle_dispute_evidence_v4", "info": {
                "submitter": key(3), "bundle_escrow": key(1),
                "bundle_verification_dispute": key(5), "instructions_sysvar": key(0),
                "verification_hash": HASH, "page_hashes": [HASH, ZERO_HASH, ZERO_HASH],
                "quorum_verifier_bitmap": 5,
            }
        }),
    );
}

#[test]
fn small_credit_claim_and_slash() {
    assert_parsed(
        SlashSmallCreditsArgs {
            amount: 123,
            sequence: 456,
        }
        .to_bytes(),
        5,
        &[3, 1, 5, 0, 4],
        json!({
            "type": "slash_small_credits", "info": {
                "slash_authority": key(3), "config_policy": key(1), "mint": key(5),
                "token_account": key(0), "token_program": key(4), "amount": 123, "sequence": 456,
            }
        }),
    );
    assert_parsed(
        ClaimSmallCreditsV4Args {}.to_bytes(),
        5,
        &[3, 1, 5, 0, 4],
        json!({
            "type": "claim_small_credits_v4", "info": {
                "bundle_escrow": key(3), "config_policy": key(1), "mint": key(5),
                "token_account": key(0), "token_program": key(4),
            }
        }),
    );
}

#[test]
fn open_escrows() {
    assert_parsed(
        OpenBundleEscrowV4Args {
            bundle_version: 4,
            reward_tier: 2,
            bundle_hash: [1; 32],
            coordinator: [6; 32],
            requester_refund_recipient: [7; 32],
            total_input_tokens: 101,
            max_output_tokens: 202,
            escrow_lamports: 303,
            expected_page_count: 2,
            ..OpenBundleEscrowV4Args::zeroed()
        }
        .to_bytes(),
        4,
        &[3, 1, 5, 0],
        json!({
            "type": "open_bundle_escrow_v4", "info": {
                "payer": key(3), "bundle_escrow": key(1), "config_policy": key(5), "system_program": key(0),
                "bundle_version": 4, "reward_tier": {"name": "pro", "value": 2},
                "bundle_hash": HASH, "coordinator": key(6), "requester_refund_recipient": key(7),
                "total_input_tokens": 101, "max_output_tokens": 202, "escrow_lamports": 303, "expected_page_count": 2,
            }
        }),
    );
    assert_parsed(
        OpenPricedBundleEscrowV4Args {
            bundle_version: 5,
            reward_tier: 3,
            bundle_hash: [1; 32],
            coordinator: [6; 32],
            requester_refund_recipient: [7; 32],
            total_input_tokens: 111,
            max_output_tokens: 222,
            escrow_lamports: 333,
            expected_page_count: 3,
            pricing_commitment: [1; 32],
            ..OpenPricedBundleEscrowV4Args::zeroed()
        }
        .to_bytes(),
        4,
        &[3, 1, 5, 0],
        json!({
            "type": "open_priced_bundle_escrow_v4", "info": {
                "payer": key(3), "bundle_escrow": key(1), "config_policy": key(5), "system_program": key(0),
                "bundle_version": 5, "reward_tier": {"name": "small", "value": 3},
                "bundle_hash": HASH, "coordinator": key(6), "requester_refund_recipient": key(7),
                "total_input_tokens": 111, "max_output_tokens": 222, "escrow_lamports": 333,
                "expected_page_count": 3, "pricing_commitment": HASH,
            }
        }),
    );
    assert_parsed(
        OpenSmallBundleEscrowV4Args {
            bundle_version: 6,
            bundle_hash: [1; 32],
            coordinator: [6; 32],
            requester_refund_recipient: [7; 32],
            total_input_tokens: 11,
            max_output_tokens: 22,
            expected_page_count: 1,
            winner_node_pubkey: [4; 32],
            ..OpenSmallBundleEscrowV4Args::zeroed()
        }
        .to_bytes(),
        4,
        &[3, 1, 5, 0],
        json!({
            "type": "open_small_bundle_escrow_v4", "info": {
                "payer": key(3), "bundle_escrow": key(1), "config_policy": key(5), "system_program": key(0),
                "bundle_version": 6, "bundle_hash": HASH, "coordinator": key(6), "requester_refund_recipient": key(7),
                "total_input_tokens": 11, "max_output_tokens": 22, "expected_page_count": 1, "winner_node_pubkey": key(4),
            }
        }),
    );
}

#[test]
fn pricing_and_page_cleanup() {
    let mut post = PostBundlePricingArgs::zeroed();
    post.page_index = 2;
    post.pricing_entry_count = 1;
    post.pricing_entries[0] = BundleJobPricingV4 {
        job_id: [7; 32].into(),
        max_output_tokens: 123,
        price_per_output_token: 456,
    };
    assert_parsed(
        post.to_bytes(),
        3,
        &[3, 1, 5],
        json!({
            "type": "post_bundle_pricing", "info": {
                "coordinator": key(3), "bundle_escrow": key(1), "bundle_verifier_page": key(5),
                "page_index": 2, "pricing_entry_count": 1,
                "pricing_entries": [{"job_id": key(7), "max_output_tokens": 123, "price_per_output_token": 456}],
            }
        }),
    );
    for pages in [false, true] {
        let accounts = if pages {
            &[3, 1, 5, 0][..]
        } else {
            &[3, 1][..]
        };
        assert_parsed(
            SealBundlePricingArgs::zeroed().to_bytes(),
            2,
            accounts,
            json!({
                "type": "seal_bundle_pricing", "info": {
                    "coordinator": key(3), "bundle_escrow": key(1),
                    "bundle_verifier_pages": if pages { vec![key(5), key(0)] } else { vec![] },
                }
            }),
        );
    }
    assert_parsed(
        CloseBundleVerifierPageV4Args {
            page_index: 2,
            disputed: 1,
            _reserved: [0; 5],
        }
        .to_bytes(),
        3,
        &[3, 1, 5],
        json!({
            "type": "close_bundle_verifier_page_v4", "info": {
                "funder": key(3), "bundle_escrow": key(1), "bundle_verifier_page": key(5),
                "page_index": 2, "disputed": 1,
            }
        }),
    );
}

#[test]
fn settlement_commits() {
    assert_parsed(
        CommitAuctionSettlementV3Args {
            auction_hash: [1; 32],
            winner_node_pubkey: [7; 32],
        }
        .to_bytes(),
        4,
        &[3, 1, 5, 0],
        json!({
            "type": "commit_auction_settlement_v3", "info": {
                "coordinator": key(3), "bundle_escrow": key(1), "config_policy": key(5),
                "winner_vote_account": key(0), "auction_hash": HASH, "winner_node_pubkey": key(7),
            }
        }),
    );
    assert_parsed(
        CommitSmallAuctionSettlementV4Args {
            auction_hash: [1; 32],
        }
        .to_bytes(),
        3,
        &[3, 1, 5],
        json!({
            "type": "commit_small_auction_settlement_v4", "info": {
                "coordinator": key(3), "bundle_escrow": key(1), "config_policy": key(5), "auction_hash": HASH,
            }
        }),
    );
}

fn tier_config() -> (RequestTierConfigV2, Value) {
    (
        RequestTierConfigV2 {
            bid_reveal_duration: 1,
            active_auction_duration: 2,
            bundle_duration: 3,
            requests_per_bundle: 4,
            max_context_length_tokens: 5,
            job_submission_duration_slots: 6,
            bid_commitment_amount_multiplier: 7,
            auction_credits_multiplier: 8,
            settlement_window_slots: 9,
            result_window_slots: 10,
            verification_window_slots: 11,
            claim_window_slots: 12,
        },
        json!({
            "bid_reveal_duration": 1, "active_auction_duration": 2, "bundle_duration": 3,
            "requests_per_bundle": 4, "max_context_length_tokens": 5, "job_submission_duration_slots": 6,
            "bid_commitment_amount_multiplier": 7, "auction_credits_multiplier": 8,
            "settlement_window_slots": 9, "result_window_slots": 10, "verification_window_slots": 11,
            "claim_window_slots": 12,
        }),
    )
}

#[test]
fn historical_and_current_policy_initialization() {
    let (tier, tier_json) = tier_config();
    let historical = InitConfigPolicySmallV3Args {
        config_policy_lamports: 1234,
        initial_admin_authority: [6; 32].into(),
        service_authority: [7; 32].into(),
        minimum_bundle_auction_pairs: 23,
        max_auction_credits_per_update: 456,
        v2_verifiers_per_auction: 3,
        v2_verifier_quorum: 2,
        tier_configs: [tier; 5],
        ..InitConfigPolicySmallV3Args::zeroed()
    };
    let current = InitConfigPolicyV2Args {
        config_policy_lamports: historical.config_policy_lamports,
        initial_admin_authority: historical.initial_admin_authority,
        service_authority: historical.service_authority,
        minimum_bundle_auction_pairs: historical.minimum_bundle_auction_pairs,
        max_auction_credits_per_update: historical.max_auction_credits_per_update,
        v2_verifiers_per_auction: 3,
        v2_verifier_quorum: 2,
        tier_configs: [tier; 5],
        missed_verification_dispute_window_slots: 31,
        dispute_verification_window_slots: 32,
        paid_verification_dispute_window_slots: 33,
        paid_verification_dispute_bond_lamports: 34,
        ..InitConfigPolicyV2Args::zeroed()
    };
    assert_eq!(historical.to_bytes().len(), 585);
    assert_eq!(current.to_bytes().len(), 617);
    let mut expected = json!({"type": "init_config_policy_v2", "info": {
        "authority": key(3), "config_policy": key(1), "system_program": key(5),
        "config_policy_lamports": 1234, "initial_admin_authority": key(6), "service_authority": key(7),
        "policy_flags": {"bits": 0, "names": []}, "minimum_bundle_auction_pairs": 23,
        "max_auction_credits_per_update": 456, "v2_verifiers_per_auction": 3, "v2_verifier_quorum": 2,
        "tier_configs": vec![tier_json; 5],
    }});
    assert_parsed(historical.to_bytes(), 3, &[3, 1, 5], expected.clone());
    expected["info"]["missed_verification_dispute_window_slots"] = json!(31);
    expected["info"]["dispute_verification_window_slots"] = json!(32);
    expected["info"]["paid_verification_dispute_window_slots"] = json!(33);
    expected["info"]["paid_verification_dispute_bond_lamports"] = json!(34);
    assert_parsed(current.to_bytes(), 3, &[3, 1, 5], expected);
}

#[test]
fn historical_and_current_policy_updates() {
    let (tier, tier_json) = tier_config();
    let mut historical = SetConfigPolicySmallV3Args {
        patch_kind: ConfigPolicyV2PatchKind::AUTHORITY,
        authority_index: 4,
        authority: [7; 32].into(),
        tier: 2,
        tier_config: tier,
        policy_flags: ConfigPolicyV2Flags::from_flag(
            ambient_auction_api::ConfigPolicyV2Flag::AllowServiceCommitOverride,
        ),
        max_auction_credits_per_update: 456,
        v2_verifiers_per_auction: 3,
        v2_verifier_quorum: 2,
        ..SetConfigPolicySmallV3Args::zeroed()
    };
    let mut expected = json!({"type": "set_config_policy_v2", "info": {
        "authority": key(3), "config_policy": key(1),
        "patch_kind": {"name": "authority", "value": 1}, "authority_kind": {"name": "admin", "value": 0},
        "authority_index": 4, "authority_pubkey": key(7), "tier": {"name": "pro", "value": 2},
        "tier_config": tier_json, "policy_flags": {"bits": 2, "names": ["allow_service_commit_override"]},
        "max_auction_credits_per_update": 456, "v2_verifiers_per_auction": 3, "v2_verifier_quorum": 2,
    }});
    assert_eq!(historical.to_bytes().len(), 161);
    assert_parsed(historical.to_bytes(), 2, &[3, 1], expected.clone());
    let current = SetConfigPolicyV2Args {
        patch_kind: ConfigPolicyV2PatchKind::DISPUTE_SETTINGS,
        authority_index: 4,
        authority: historical.authority,
        tier: 2,
        tier_config: tier,
        policy_flags: historical.policy_flags,
        max_auction_credits_per_update: 456,
        v2_verifiers_per_auction: 3,
        v2_verifier_quorum: 2,
        missed_verification_dispute_window_slots: 31,
        dispute_verification_window_slots: 32,
        paid_verification_dispute_window_slots: 33,
        paid_verification_dispute_bond_lamports: 34,
        ..SetConfigPolicyV2Args::zeroed()
    };
    assert_eq!(current.to_bytes().len(), 193);
    let mut current_expected = expected.clone();
    current_expected["info"]["patch_kind"] = json!({"name": "dispute_settings", "value": 5});
    current_expected["info"]["missed_verification_dispute_window_slots"] = json!(31);
    current_expected["info"]["dispute_verification_window_slots"] = json!(32);
    current_expected["info"]["paid_verification_dispute_window_slots"] = json!(33);
    current_expected["info"]["paid_verification_dispute_bond_lamports"] = json!(34);
    assert_parsed(current.to_bytes(), 2, &[3, 1], current_expected);
    for (kind, name) in [
        (6, "small_credit_slash_authority"),
        (7, "small_credit_settings"),
        (200, "unknown"),
    ] {
        historical.patch_kind = ConfigPolicyV2PatchKind(kind);
        historical.small_credit_enabled = 1;
        expected["info"]["patch_kind"] = json!({"name": name, "value": kind});
        if kind == 7 {
            expected["info"]["small_credit_enabled"] = json!(1);
        } else {
            expected["info"]
                .as_object_mut()
                .unwrap()
                .remove("small_credit_enabled");
        }
        assert_parsed(historical.to_bytes(), 2, &[3, 1], expected.clone());
    }
}

#[test]
fn historical_and_extended_results_with_optional_accounts() {
    let mut post = PostBundleResultV2Args {
        result_hash: [1; 32],
        posted_output_tokens: 123,
        page_index: 2,
        page_entry_count: 1,
        ..PostBundleResultV2Args::zeroed()
    };
    post.page_entries[0] = BundleVerifierPageV2Entry {
        job_id: [7; 32].into(),
        posted_output_tokens: 23,
        accepted_output_tokens: 17,
        assigned_verifiers_token_ranges: [0, 7, 7, 17, 0, 0],
        verifier_reward_tokens: [7, 10, 0],
        verdict: VerificationVerdictV2::Verified,
        verifier_claimed_bitmap: 1,
        _reserved: [0; 6],
    };
    for extended in [false, true] {
        let data = if extended {
            PostBundleResultV3Args {
                post,
                input_tokens: [31, 0, 0, 0, 0, 0],
            }
            .to_bytes()
        } else {
            post.to_bytes()
        };
        for count in 3..=5 {
            let mut expected = json!({"type": "post_bundle_result_v2", "info": {
                "authority": key(3), "bundle_escrow": key(1), "config_policy": key(5),
                "bundle_verifier_page": if count > 3 { Some(key(0)) } else { None },
                "result_hash": HASH, "posted_output_tokens": 123, "page_index": 2, "page_entry_count": 1,
                "page_entries": [{"job_id": key(7), "posted_output_tokens": 23, "accepted_output_tokens": 17,
                    "assigned_verifiers_token_ranges": [0, 7, 7, 17, 0, 0], "verifier_reward_tokens": [7, 10, 0],
                    "verdict": {"name": "verified", "value": 1}, "verifier_claimed_bitmap": 1}],
            }});
            if extended {
                expected["info"]["input_tokens"] = json!([31, 0, 0, 0, 0, 0]);
            }
            if count == 5 {
                expected["info"]["bundle_verification_dispute"] = json!(key(4));
            }
            assert_parsed(data.clone(), 3, &[3, 1, 5, 0, 4][..count], expected);
        }
    }
}

#[test]
fn unsupported_instructions_return_errors() {
    for data in [vec![24, 1, 2], vec![37], vec![255]] {
        assert_eq!(
            parse_instruction(&data, &[[7; 32]]),
            Err(ParseInstructionError::InstructionKeyMismatch)
        );
    }
    assert_eq!(
        parse_instruction(&[11], &[[7; 32]]),
        Err(ParseInstructionError::InstructionNotParsable)
    );
}

#[test]
fn empty_data_returns_an_error() {
    for accounts in [vec![], vec![[0; 32]]] {
        assert_eq!(
            parse_instruction(&[], &accounts),
            Err(ParseInstructionError::InstructionNotParsable)
        );
    }
}

#[test]
fn append_data_names_and_requires_all_three_accounts() {
    let accounts = [[2; 32], [0; 32], [1; 32]];
    let parsed = parse_instruction(&[10], &accounts).unwrap();
    assert_eq!(parsed.instruction_type, "append_data");
    assert_eq!(
        parsed.info,
        json!({"data_authority": key(2), "data_account": key(0), "system_program": key(1)})
    );
    assert_eq!(
        parse_instruction(&[10], &accounts[..2]),
        Err(ParseInstructionError::InstructionKeyMismatch)
    );
}

const THREES: &str = "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM=";
const FOURS: &str = "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ=";
const FIVES: &str = "BQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQU=";

#[test]
fn replay_anchor_and_replay_policy_patch() {
    let anchor = AnchorReplaySpansV1Args {
        engine: [3; 32],
        span_id: 42,
        commit_sha256: [4; 32],
    };
    assert_parsed(
        anchor.to_bytes(),
        4,
        &[1, 2, 3, 4],
        json!({"type": "anchor_replay_spans_v1", "info": {
            "miner_node": key(1), "payer": key(2), "replay_anchor": key(3), "system_program": key(4),
            "engine": THREES, "span_id": 42, "commit_sha256": FOURS,
        }}),
    );
    let mut replay = ReplayPolicyV1 {
        class_id: [5; 32],
        version: 1,
        tier_mask: 2,
        arbiters: 3,
        hold_slots: 900,
        forfeit_mask: 6,
        ..ReplayPolicyV1::default()
    };
    replay.suspension_hours[0] = 24;
    let set = SetConfigPolicyReplayV1Args {
        patch_kind: ConfigPolicyV2PatchKind::REPLAY_SETTINGS,
        _reserved0: [0; 7],
        replay,
    };
    let mut suspension_hours = vec![0; replay.suspension_hours.len()];
    suspension_hours[0] = 24;
    assert_parsed(
        set.to_bytes(),
        2,
        &[3, 1],
        json!({"type": "set_config_policy_v2", "info": {
            "authority": key(3), "config_policy": key(1),
            "patch_kind": {"name": "replay_settings", "value": 8},
            "replay": {"class_id": FIVES, "version": 1, "tier_mask": 2, "arbiters": 3,
                "hold_slots": 900, "forfeit_mask": 6, "suspension_hours": suspension_hours},
        }}),
    );
}
