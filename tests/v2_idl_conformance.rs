#![cfg(feature = "idl")]

use ambient_auction_api::{
    idl as schema,
    state::{
        AccountHeaderV1, BundleEscrowV2ReservedData, BundleVerificationDisputeV2Kind,
        BundleVerifierPageV2Entry, ConfigPolicyV2, ConfigPolicyV2Flag, ConfigPolicyV2Flags, Pubkey,
        RawBundleEscrowV2Data,
        RawBundleVerificationDisputeV2Data, RawBundleVerifierPageV2Data, RequestTierConfigV2,
    },
    AuctionInstruction, CommitAuctionSettlementV2Args, DisputeBundleVerificationV2Args,
    FinalizeBundleVerificationV2Args, InitBundleVerifierPageV2Args, InitConfigPolicyV2Args,
    InstructionBytes, OpenBundleEscrowV2Args, PostBundleResultV2Args, SetConfigPolicyV2Args,
    VerificationVerdictV2,
};
use borsh::BorshSerialize;
use memoffset::offset_of;
use serde_json::{json, Value};
use std::mem::size_of;

const SHANK_IDL: &str = include_str!("../idl/ambient_auction_v2.json");
const CODAMA_IDL: &str = include_str!("../codama/ambient_auction_v2.json");

fn shank() -> Value {
    serde_json::from_str(SHANK_IDL).unwrap()
}

fn definition<'a>(idl: &'a Value, name: &str) -> &'a Value {
    idl["types"]
        .as_array()
        .unwrap()
        .iter()
        .chain(idl["accounts"].as_array().unwrap())
        .find(|definition| definition["name"] == name)
        .unwrap_or_else(|| panic!("missing IDL definition {name}"))
}

fn type_size(idl: &Value, ty: &Value) -> usize {
    if let Some(primitive) = ty.as_str() {
        return match primitive {
            "u8" => 1,
            "u16" => 2,
            "u32" => 4,
            "u64" => 8,
            "publicKey" => 32,
            other => panic!("unsupported IDL primitive {other}"),
        };
    }
    if let Some(array) = ty.get("array") {
        return type_size(idl, &array[0]) * array[1].as_u64().unwrap() as usize;
    }
    if let Some(name) = ty.get("defined").and_then(Value::as_str) {
        return definition_size(idl, name);
    }
    panic!("unsupported IDL type {ty}");
}

fn definition_size(idl: &Value, name: &str) -> usize {
    definition(idl, name)["type"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| type_size(idl, &field["type"]))
        .sum()
}

fn field_offset(idl: &Value, type_name: &str, field_name: &str) -> usize {
    let fields = definition(idl, type_name)["type"]["fields"]
        .as_array()
        .unwrap();
    fields
        .iter()
        .take_while(|field| field["name"] != field_name)
        .map(|field| type_size(idl, &field["type"]))
        .sum()
}

fn assert_encoding<T: InstructionBytes, U: BorshSerialize>(actual: &T, described: &U) {
    let mut encoded = vec![T::INSTRUCTION as u8];
    described.serialize(&mut encoded).unwrap();
    assert_eq!(encoded, actual.to_bytes());
}

fn actual_tier(seed: u64) -> RequestTierConfigV2 {
    RequestTierConfigV2 {
        bid_reveal_duration: seed,
        active_auction_duration: seed + 1,
        bundle_duration: seed + 2,
        requests_per_bundle: seed + 3,
        max_context_length_tokens: seed + 4,
        job_submission_duration_slots: seed + 5,
        bid_commitment_amount_multiplier: seed + 6,
        auction_credits_multiplier: seed + 7,
        settlement_window_slots: seed + 8,
        result_window_slots: seed + 9,
        verification_window_slots: seed + 10,
        claim_window_slots: seed + 11,
    }
}

fn described_tier(seed: u64) -> schema::RequestTierConfigV2 {
    schema::RequestTierConfigV2 {
        bid_reveal_duration: seed,
        active_auction_duration: seed + 1,
        bundle_duration: seed + 2,
        requests_per_bundle: seed + 3,
        max_context_length_tokens: seed + 4,
        job_submission_duration_slots: seed + 5,
        bid_commitment_amount_multiplier: seed + 6,
        auction_credits_multiplier: seed + 7,
        settlement_window_slots: seed + 8,
        result_window_slots: seed + 9,
        verification_window_slots: seed + 10,
        claim_window_slots: seed + 11,
    }
}

fn actual_entry(seed: u8) -> BundleVerifierPageV2Entry {
    BundleVerifierPageV2Entry {
        job_id: [seed; 32].into(),
        posted_output_tokens: u64::MAX - u64::from(seed),
        accepted_output_tokens: u64::from(seed),
        assigned_verifiers_token_ranges: std::array::from_fn(|index| {
            u64::from(seed) * 10 + index as u64
        }),
        verifier_reward_tokens: std::array::from_fn(|index| u64::from(seed) * 100 + index as u64),
        verdict: if seed % 2 == 0 {
            VerificationVerdictV2::Verified
        } else {
            VerificationVerdictV2::Rejected
        },
        verifier_claimed_bitmap: seed,
        _reserved: [0; 6],
    }
}

fn described_entry(seed: u8) -> schema::BundleVerifierPageV2Entry {
    schema::BundleVerifierPageV2Entry {
        job_id: [seed; 32],
        posted_output_tokens: u64::MAX - u64::from(seed),
        accepted_output_tokens: u64::from(seed),
        assigned_verifiers_token_ranges: std::array::from_fn(|index| {
            u64::from(seed) * 10 + index as u64
        }),
        verifier_reward_tokens: std::array::from_fn(|index| u64::from(seed) * 100 + index as u64),
        verdict: if seed % 2 == 0 { 1 } else { 2 },
        verifier_claimed_bitmap: seed,
        reserved: [0; 6],
    }
}

#[test]
fn generated_instruction_interface_matches_v2_abi() {
    let idl = shank();
    let expected = [
        ("OpenBundleEscrowV2", 12, 4),
        ("CommitAuctionSettlementV2", 13, 4),
        ("PostBundleResultV2", 14, 4),
        ("FinalizeBundleVerificationV2", 15, 6),
        ("ClaimWinnerLstakeV2", 16, 5),
        ("ClaimVerifierLstakeV2", 17, 5),
        ("ExpireBundleEscrowV2", 18, 3),
        ("InitConfigPolicyV2", 19, 3),
        ("SetConfigPolicyV2", 20, 2),
        ("InitBundleVerifierPageV2", 21, 4),
        ("DisputeBundleVerificationV2", 22, 6),
        ("SelectBundleVerifiersV2", 23, 2),
    ];
    let instructions = idl["instructions"].as_array().unwrap();
    assert_eq!(instructions.len(), expected.len());
    for ((name, discriminator, account_count), instruction) in
        expected.into_iter().zip(instructions)
    {
        assert_eq!(instruction["name"], name);
        assert_eq!(instruction["discriminant"]["value"], discriminator);
        assert_eq!(
            instruction["accounts"].as_array().unwrap().len(),
            account_count
        );
        assert_eq!(
            AuctionInstruction::try_from(discriminator).unwrap() as u8,
            discriminator
        );
    }

    let post_page = &instructions[2]["accounts"][3];
    assert_eq!(post_page["isMut"], true);
    assert_eq!(post_page["isOptional"], true);
    let select_dispute = &instructions[11]["accounts"][1];
    assert_eq!(select_dispute["isMut"], true);
    assert_eq!(select_dispute["isOptional"], true);
    assert_eq!(
        instructions[3]["accounts"][4],
        json!({
            "name": "instructionsSysvar", "isMut": false, "isSigner": false
        })
    );
}

#[test]
fn idl_argument_encodings_match_pod_edge_cases() {
    assert_encoding(
        &OpenBundleEscrowV2Args {
            bundle_version: u32::MAX,
            _reserved0: [0; 4],
            reward_tier: u64::MAX,
            bundle_hash: [0xa5; 32],
            coordinator: [0x11; 32],
            requester_refund_recipient: [0x22; 32],
            total_input_tokens: 1,
            max_output_tokens: u64::MAX - 1,
            escrow_lamports: 1 << 63,
        },
        &schema::OpenBundleEscrowV2Args {
            bundle_version: u32::MAX,
            reserved0: [0; 4],
            reward_tier: u64::MAX,
            bundle_hash: [0xa5; 32],
            coordinator: [0x11; 32],
            requester_refund_recipient: [0x22; 32],
            total_input_tokens: 1,
            max_output_tokens: u64::MAX - 1,
            escrow_lamports: 1 << 63,
        },
    );
    assert_encoding(
        &CommitAuctionSettlementV2Args {
            auction_hash: [0x33; 32],
            winner_node_pubkey: [0x44; 32],
            clearing_price_per_output_token: u64::MAX,
        },
        &schema::CommitAuctionSettlementV2Args {
            auction_hash: [0x33; 32],
            winner_node_pubkey: [0x44; 32],
            clearing_price_per_output_token: u64::MAX,
        },
    );
    assert_encoding(
        &PostBundleResultV2Args {
            result_hash: [0x55; 32],
            posted_output_tokens: u64::MAX,
            page_index: u16::MAX,
            page_entry_count: 6,
            _reserved: [0; 4],
            page_entries: std::array::from_fn(|index| actual_entry(index as u8)),
        },
        &schema::PostBundleResultV2Args {
            result_hash: [0x55; 32],
            posted_output_tokens: u64::MAX,
            page_index: u16::MAX,
            page_entry_count: 6,
            reserved: [0; 4],
            page_entries: std::array::from_fn(|index| described_entry(index as u8)),
        },
    );
    assert_encoding(
        &FinalizeBundleVerificationV2Args {
            verification_hash: [0x66; 32],
            accepted_output_tokens: u64::MAX,
            winner_payout_lamports: 1 << 63,
            verdict: VerificationVerdictV2::Rejected,
            quorum_verifier_bitmap: 0b101,
            _reserved: [0; 6],
        },
        &schema::FinalizeBundleVerificationV2Args {
            verification_hash: [0x66; 32],
            accepted_output_tokens: u64::MAX,
            winner_payout_lamports: 1 << 63,
            verdict: 2,
            quorum_verifier_bitmap: 0b101,
            reserved: [0; 6],
        },
    );

    let flags = ConfigPolicyV2Flags::from_flag(ConfigPolicyV2Flag::AllowServiceFinalizeOverride);
    assert_encoding(
        &InitConfigPolicyV2Args {
            config_policy_lamports: u64::MAX,
            initial_admin_authority: Pubkey::from([0x77; 32]),
            service_authority: Pubkey::from([0x88; 32]),
            policy_flags: flags,
            minimum_bundle_auction_pairs: 2,
            max_auction_credits_per_update: u64::MAX - 1,
            v2_verifiers_per_auction: 3,
            v2_verifier_quorum: 2,
            _reserved0: [0; 6],
            missed_verification_dispute_window_slots: 1,
            dispute_verification_window_slots: 2,
            paid_verification_dispute_window_slots: 3,
            paid_verification_dispute_bond_lamports: 4,
            tier_configs: std::array::from_fn(|index| actual_tier(10 + index as u64 * 20)),
        },
        &schema::InitConfigPolicyV2Args {
            config_policy_lamports: u64::MAX,
            initial_admin_authority: [0x77; 32],
            service_authority: [0x88; 32],
            policy_flags: flags.bits(),
            minimum_bundle_auction_pairs: 2,
            max_auction_credits_per_update: u64::MAX - 1,
            v2_verifiers_per_auction: 3,
            v2_verifier_quorum: 2,
            reserved0: [0; 6],
            missed_verification_dispute_window_slots: 1,
            dispute_verification_window_slots: 2,
            paid_verification_dispute_window_slots: 3,
            paid_verification_dispute_bond_lamports: 4,
            tier_configs: std::array::from_fn(|index| described_tier(10 + index as u64 * 20)),
        },
    );

    let set = SetConfigPolicyV2Args {
        patch_kind: ambient_auction_api::ConfigPolicyV2PatchKind::TIER_CONFIG,
        authority_kind: ambient_auction_api::ConfigPolicyV2AuthorityKind::SERVICE,
        authority_index: u8::MAX,
        v2_verifiers_per_auction: 3,
        v2_verifier_quorum: 2,
        _reserved0: [0; 3],
        tier: u64::MAX,
        policy_flags: flags,
        max_auction_credits_per_update: u64::MAX - 2,
        missed_verification_dispute_window_slots: 11,
        dispute_verification_window_slots: 12,
        paid_verification_dispute_window_slots: 13,
        paid_verification_dispute_bond_lamports: 14,
        authority: Pubkey::from([0x99; 32]),
        tier_config: actual_tier(100),
    };
    assert_encoding(
        &set,
        &schema::SetConfigPolicyV2Args {
            patch_kind: set.patch_kind.0,
            authority_kind: set.authority_kind.0,
            authority_index: set.authority_index,
            v2_verifiers_per_auction: set.v2_verifiers_per_auction,
            v2_verifier_quorum: set.v2_verifier_quorum,
            reserved0: [0; 3],
            tier: set.tier,
            policy_flags: set.policy_flags.bits(),
            max_auction_credits_per_update: set.max_auction_credits_per_update,
            missed_verification_dispute_window_slots: set.missed_verification_dispute_window_slots,
            dispute_verification_window_slots: set.dispute_verification_window_slots,
            paid_verification_dispute_window_slots: set.paid_verification_dispute_window_slots,
            paid_verification_dispute_bond_lamports: set.paid_verification_dispute_bond_lamports,
            authority: set.authority.inner(),
            tier_config: described_tier(100),
        },
    );
    assert_encoding(
        &InitBundleVerifierPageV2Args {
            bundle_verifier_page_lamports: u64::MAX,
            page_index: u16::MAX,
            _reserved: [0; 6],
        },
        &schema::InitBundleVerifierPageV2Args {
            bundle_verifier_page_lamports: u64::MAX,
            page_index: u16::MAX,
            reserved: [0; 6],
        },
    );
    assert_encoding(
        &DisputeBundleVerificationV2Args {
            kind: BundleVerificationDisputeV2Kind::PaidVerdictDispute,
            _reserved: [0; 7],
        },
        &schema::DisputeBundleVerificationV2Args {
            kind: 2,
            reserved: [0; 7],
        },
    );
}

#[test]
fn generated_account_sizes_and_offsets_match_pod_layouts() {
    let idl = shank();
    let codama: Value = serde_json::from_str(CODAMA_IDL).unwrap();
    let account_sizes = codama["program"]["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|account| {
            (
                account["name"].as_str().unwrap(),
                account["size"].as_u64().unwrap(),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(
        account_sizes["bundleEscrowV2"] as usize,
        RawBundleEscrowV2Data::LEN_V2
    );
    assert_eq!(
        account_sizes["bundleVerifierPageV2"] as usize,
        RawBundleVerifierPageV2Data::LEN_V2
    );
    assert_eq!(
        account_sizes["bundleVerificationDisputeV2"] as usize,
        RawBundleVerificationDisputeV2Data::LEN
    );
    assert_eq!(
        account_sizes["configPolicyV2"] as usize,
        ConfigPolicyV2::LEN
    );

    assert_eq!(
        definition_size(&idl, "AccountHeaderV1"),
        size_of::<AccountHeaderV1>()
    );
    assert_eq!(
        definition_size(&idl, "RawBundleEscrowV2Data"),
        size_of::<RawBundleEscrowV2Data>()
    );
    assert_eq!(
        definition_size(&idl, "BundleEscrowV2ReservedData"),
        size_of::<BundleEscrowV2ReservedData>()
    );
    assert_eq!(
        definition_size(&idl, "BundleVerifierPageV2Entry"),
        size_of::<BundleVerifierPageV2Entry>()
    );
    assert_eq!(
        definition_size(&idl, "RawBundleVerifierPageV2Data"),
        size_of::<RawBundleVerifierPageV2Data>()
    );
    assert_eq!(
        definition_size(&idl, "RawBundleVerificationDisputeV2Data"),
        size_of::<RawBundleVerificationDisputeV2Data>()
    );
    assert_eq!(
        definition_size(&idl, "RequestTierConfigV2"),
        size_of::<RequestTierConfigV2>()
    );
    assert_eq!(
        definition_size(&idl, "ConfigPolicyV2"),
        size_of::<ConfigPolicyV2>()
    );

    assert_eq!(
        field_offset(&idl, "RawBundleEscrowV2Data", "bundleVersion"),
        offset_of!(RawBundleEscrowV2Data, bundle_version)
    );
    assert_eq!(
        field_offset(&idl, "RawBundleEscrowV2Data", "selectedVerifiers"),
        offset_of!(RawBundleEscrowV2Data, selected_verifiers)
    );
    assert_eq!(
        field_offset(&idl, "RawBundleEscrowV2Data", "winnerRewardClaimed"),
        offset_of!(RawBundleEscrowV2Data, winner_reward_claimed)
    );
    assert_eq!(
        field_offset(&idl, "RawBundleEscrowV2Data", "verifierRewardRemaining"),
        offset_of!(RawBundleEscrowV2Data, verifier_reward_remaining)
    );
    assert_eq!(
        field_offset(&idl, "BundleEscrowV2ReservedData", "winnerAuctionCredits"),
        offset_of!(BundleEscrowV2ReservedData, winner_auction_credits)
    );
    assert_eq!(
        field_offset(&idl, "BundleVerifierPageV2Entry", "verdict"),
        offset_of!(BundleVerifierPageV2Entry, verdict)
    );
    assert_eq!(
        field_offset(&idl, "RawBundleVerifierPageV2Data", "entries"),
        offset_of!(RawBundleVerifierPageV2Data, entries)
    );
    assert_eq!(
        field_offset(
            &idl,
            "RawBundleVerificationDisputeV2Data",
            "replacementVerifiers"
        ),
        offset_of!(RawBundleVerificationDisputeV2Data, replacement_verifiers)
    );
    assert_eq!(
        field_offset(
            &idl,
            "RawBundleVerificationDisputeV2Data",
            "replacementDeadlineSlot"
        ),
        offset_of!(
            RawBundleVerificationDisputeV2Data,
            replacement_deadline_slot
        )
    );
    assert_eq!(
        field_offset(&idl, "ConfigPolicyV2", "tierConfigs"),
        offset_of!(ConfigPolicyV2, tier_configs)
    );
    assert_eq!(
        field_offset(&idl, "ConfigPolicyV2", "v2AccountLayoutVersion"),
        offset_of!(ConfigPolicyV2, v2_account_layout_version)
    );
    assert_eq!(
        field_offset(&idl, "ConfigPolicyV2", "reservedTail"),
        offset_of!(ConfigPolicyV2, reserved_tail)
    );
}
