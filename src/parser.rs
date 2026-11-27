//! Auction instruction decoding and JSON formatting for off-chain callers.

use {
    crate::instruction::{
        AnchorReplaySpansV1Args, AuctionInstruction, AuthorizeBundleDisputeEvidenceV4Args,
        ClaimSmallCreditsV4Args, ClaimVerifierLstakeV2Args, ClaimWinnerLstakeV2Args,
        CloseBundleVerifierPageV4Args, CommitAuctionSettlementV2Args,
        CommitAuctionSettlementV3Args, CommitSmallAuctionSettlementV4Args,
        DisputeBundleVerificationV2Args, ExpireBundleEscrowV2Args,
        FinalizeBundleVerificationV2Args, InitBundleVerifierPageV2Args,
        InitConfigPolicySmallV3Args, InitConfigPolicyV2Args, OpenBundleEscrowV2Args,
        OpenBundleEscrowV4Args, OpenPricedBundleEscrowV4Args, OpenSmallBundleEscrowV4Args,
        PlaceBidArgs, PostBundlePricingArgs, PostBundleResultV2Args, PostBundleResultV3Args,
        RequestJobArgs, RevealBidArgs, SealBundlePricingArgs, SelectBundleVerifiersV2Args,
        SetConfigPolicyReplayV1Args, SetConfigPolicySmallV3Args, SetConfigPolicyV2Args,
        SlashSmallCreditsArgs, SubmitJobOutputArgs, SubmitValidationArgs,
    },
    crate::{
        BundleVerificationDisputeV2Kind, ConfigPolicyV2AuthorityKind, ConfigPolicyV2Flag,
        ConfigPolicyV2Flags, ConfigPolicyV2PatchKind, ReplayPolicyV1, RequestTier,
        RequestTierConfigV2, VerificationVerdictV2,
    },
    base64::{prelude::BASE64_STANDARD, Engine as _},
    bytemuck::{try_pod_read_unaligned, Zeroable},
    serde::{Deserialize, Serialize},
    serde_json::{json, Value},
    std::mem::size_of,
};

use crate::instruction::{
    JoinReplayClassV1Args, LeaveReplayClassV1Args, RemoveReplayClassMemberV1Args,
};

/// An auction instruction formatted for JSON output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParsedInstruction {
    /// The instruction name used in RPC responses.
    #[serde(rename = "type")]
    pub instruction_type: String,
    /// The named instruction arguments and account addresses.
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub info: Value,
}

/// An instruction cannot be decoded or has incompatible accounts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ParseInstructionError {
    /// The instruction data does not match a supported format.
    #[error("auction instruction not parsable")]
    InstructionNotParsable,
    /// The accounts or instruction number do not match a supported instruction.
    #[error("auction instruction key mismatch")]
    InstructionKeyMismatch,
}

/// Formats auction instruction data and resolved account addresses as JSON fields.
///
/// `data` includes the instruction number. `accounts` contains addresses in
/// instruction order, including duplicates and optional accounts.
///
/// # Errors
/// Returns an error for unsupported or malformed data, or incompatible accounts.
pub fn parse_instruction(
    data: &[u8],
    accounts: &[[u8; 32]],
) -> Result<ParsedInstruction, ParseInstructionError> {
    let discriminator = data
        .first()
        .ok_or(ParseInstructionError::InstructionNotParsable)?;
    if accounts.is_empty() {
        return Err(ParseInstructionError::InstructionKeyMismatch);
    }
    let auction_instruction: AuctionInstruction = match AuctionInstruction::try_from(*discriminator)
    {
        Ok(inst) => inst,
        Err(_) => {
            return Err(ParseInstructionError::InstructionKeyMismatch);
        }
    };
    match auction_instruction {
        AuctionInstruction::AnchorReplaySpansV1 => {
            check_num_auction_accounts(accounts, 4)?;
            let anchor = read_auction_instruction::<AnchorReplaySpansV1Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "anchor_replay_spans_v1".to_string(),
                info: json!({
                    "miner_node": pubkey(&accounts[0]),
                    "payer": pubkey(&accounts[1]),
                    "replay_anchor": pubkey(&accounts[2]),
                    "system_program": pubkey(&accounts[3]),
                    "engine": BASE64_STANDARD.encode(anchor.engine),
                    "span_id": anchor.span_id,
                    "commit_sha256": BASE64_STANDARD.encode(anchor.commit_sha256),
                }),
            })
        }
        AuctionInstruction::JoinReplayClassV1 => {
            check_num_auction_accounts(accounts, 5)?;
            let join = read_auction_instruction::<JoinReplayClassV1Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "join_replay_class_v1".to_string(),
                info: json!({
                    "node": pubkey(&accounts[0]),
                    "payer": pubkey(&accounts[1]),
                    "replay_class": pubkey(&accounts[2]),
                    "config_policy": pubkey(&accounts[3]),
                    "system_program": pubkey(&accounts[4]),
                    "class_id": BASE64_STANDARD.encode(join.class_id),
                }),
            })
        }
        AuctionInstruction::LeaveReplayClassV1 => {
            check_num_auction_accounts(accounts, 2)?;
            let leave = read_auction_instruction::<LeaveReplayClassV1Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "leave_replay_class_v1".to_string(),
                info: json!({
                    "node": pubkey(&accounts[0]),
                    "replay_class": pubkey(&accounts[1]),
                    "class_id": BASE64_STANDARD.encode(leave.class_id),
                }),
            })
        }
        AuctionInstruction::RemoveReplayClassMemberV1 => {
            check_num_auction_accounts(accounts, 3)?;
            let remove = read_auction_instruction::<RemoveReplayClassMemberV1Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "remove_replay_class_member_v1".to_string(),
                info: json!({
                    "admin": pubkey(&accounts[0]),
                    "replay_class": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "class_id": BASE64_STANDARD.encode(remove.class_id),
                    "member": auction_pubkey(remove.member),
                }),
            })
        }
        AuctionInstruction::SelectReplayVerifiersV1 => {
            check_num_auction_accounts(accounts, 5)?;
            if data.len() != 1 {
                return Err(ParseInstructionError::InstructionNotParsable);
            }
            Ok(ParsedInstruction {
                instruction_type: "select_replay_verifiers_v1".to_string(),
                info: json!({
                    "bundle_escrow": pubkey(&accounts[0]),
                    "auction_verifiers": pubkey(&accounts[1]),
                    "slot_hashes": pubkey(&accounts[2]),
                    "replay_class": pubkey(&accounts[3]),
                    "config_policy": pubkey(&accounts[4]),
                    "bundle_verification_dispute": accounts.get(5).map(pubkey),
                }),
            })
        }
        AuctionInstruction::RequestJob => {
            check_num_auction_accounts(accounts, 7)?;
            let Ok(request_job) = try_pod_read_unaligned::<RequestJobArgs>(&data[1..]) else {
                return Err(ParseInstructionError::InstructionNotParsable);
            };
            Ok(ParsedInstruction {
                instruction_type: "request_job".to_string(),
                info: json!({
                    "max_price_per_output_token": request_job.max_price_per_output_token,
                    "max_output_tokens": request_job.max_output_tokens,
                    "authority": pubkey(&request_job.authority),
                    "input_hash": BASE64_STANDARD.encode(request_job.input_hash).to_string(),
                    "input_hash_encryption_iv": if request_job.input_hash_iv != [0; 16] {
                        Some(BASE64_STANDARD.encode(request_job.input_hash_iv).to_string())
                    } else {
                        None
                    },
                    "job_request_seed": BASE64_STANDARD.encode(request_job.job_request_seed).to_string(),
                    "new_bundle_lamports": request_job.new_bundle_lamports,
                    "input_tokens": request_job.input_tokens,
                    "bump": request_job.bump,
                    "new_auction_lamports": request_job.new_auction_lamports,
                }),
            })
        }
        AuctionInstruction::PlaceBid => {
            check_num_auction_accounts(accounts, 4)?;
            let Ok(place_bid) = try_pod_read_unaligned::<PlaceBidArgs>(&data[1..]) else {
                return Err(ParseInstructionError::InstructionNotParsable);
            };
            Ok(ParsedInstruction {
                instruction_type: "place_bid".to_string(),
                info: json!({
                    "bid": pubkey(&accounts[1]),
                    "auction": pubkey(&accounts[2]),
                    "price_hash": BASE64_STANDARD.encode(place_bid.price_hash).to_string(),
                    "authority": pubkey(&place_bid.authority),
                    "ip": std::net::IpAddr::from(place_bid.ip).to_string(),
                    "port": place_bid.port,
                }),
            })
        }
        AuctionInstruction::EndAuction => {
            check_num_auction_accounts(accounts, 5)?;
            Ok(ParsedInstruction {
                instruction_type: "end_auction".to_string(),
                info: json!({
                    "auction": pubkey(&accounts[0]),
                    "bundle": pubkey(&accounts[1]),
                    "vote_account": pubkey(&accounts[2]),
                    "vote_authority": pubkey(&accounts[3]),
                    "signer": pubkey(&accounts[4]),
                }),
            })
        }
        AuctionInstruction::CloseBid => {
            check_num_auction_accounts(accounts, 5)?;
            Ok(ParsedInstruction {
                instruction_type: "close_bid".to_string(),
                info: json!({
                    "bid_authority": pubkey(&accounts[0]),
                    "bid_key": pubkey(&accounts[1]),
                    "auction_payer": pubkey(&accounts[2]),
                    "auction": pubkey(&accounts[3]),
                    "bundle": pubkey(&accounts[4]),
                }),
            })
        }
        AuctionInstruction::SubmitJobOutput => {
            check_num_auction_accounts(accounts, 3)?;
            let Ok(submit_job_output) = try_pod_read_unaligned::<SubmitJobOutputArgs>(&data[1..])
            else {
                return Err(ParseInstructionError::InstructionNotParsable);
            };
            Ok(ParsedInstruction {
                instruction_type: "submit_job_output".to_string(),
                info: json!({
                    "authority": pubkey(&accounts[0]),
                    "bundle": pubkey(&accounts[1]),
                    "job_request": pubkey(&accounts[2]),
                    "output_token_count": submit_job_output.output_token_count,
                    "input_token_count": submit_job_output.input_token_count,
                    "merkle_root": BASE64_STANDARD.encode(submit_job_output.merkle_root).to_string(),
                    "merkle_root_encryption_iv": if submit_job_output.merkle_root_iv != [0; 16] {
                        Some(BASE64_STANDARD.encode(submit_job_output.merkle_root_iv).to_string())
                    } else {
                        None
                    },
                    "output_hash": BASE64_STANDARD.encode(submit_job_output.output_hash).to_string(),
                    "output_hash_encryption_iv": if submit_job_output.output_hash_iv != [0; 16] {
                        Some(BASE64_STANDARD.encode(submit_job_output.output_hash_iv).to_string())
                    } else {
                        None
                    },
                    "encryption_node_publickey": if submit_job_output.encryption_node_publickey != [0; 32] {
                        Some(BASE64_STANDARD.encode(submit_job_output.encryption_node_publickey).to_string())
                    } else { None },
                }),
            })
        }
        AuctionInstruction::CancelBundle => {
            check_num_auction_accounts(accounts, 2)?;
            Ok(ParsedInstruction {
                instruction_type: "cancel_bundle".to_string(),
                info: json!({
                    "bundle": pubkey(&accounts[0]),
                    "signer": pubkey(&accounts[1]),
                }),
            })
        }
        AuctionInstruction::InitBundle => {
            check_num_auction_accounts(accounts, 3)?;
            Ok(ParsedInstruction {
                instruction_type: "init_bundle".to_string(),
                info: json!({
                    "payer": pubkey(&accounts[0]),
                    "bundle": pubkey(&accounts[1]),
                }),
            })
        }
        AuctionInstruction::SubmitValidation => {
            check_num_auction_accounts(accounts, 5)?;
            let Ok(submit_validation) = try_pod_read_unaligned::<SubmitValidationArgs>(&data[1..])
            else {
                return Err(ParseInstructionError::InstructionNotParsable);
            };
            Ok(ParsedInstruction {
                instruction_type: "submit_validation".to_string(),
                info: json!({
                    "bundle": pubkey(&accounts[0]),
                    "vote_account": pubkey(&accounts[1]),
                    "vote_authority": pubkey(&accounts[3]),
                    "job_request": pubkey(&accounts[4]),
                    "num_successes": submit_validation.num_successes,
                    "num_failures": submit_validation.num_failures
                }),
            })
        }
        AuctionInstruction::RevealBid => {
            check_num_auction_accounts(accounts, 6)?;
            let Ok(reveal_bid) = try_pod_read_unaligned::<RevealBidArgs>(&data[1..]) else {
                return Err(ParseInstructionError::InstructionNotParsable);
            };
            Ok(ParsedInstruction {
                instruction_type: "reveal_bid".to_string(),
                info: json!({
                    "bidder": pubkey(&accounts[0]),
                    "bid": pubkey(&accounts[1]),
                    "auction": pubkey(&accounts[2]),
                    "bundle": pubkey(&accounts[3]),
                    "vote_account": pubkey(&accounts[4]),
                    "vote_authority": pubkey(&accounts[5]),
                    "price_per_output_token": reveal_bid.price_per_output_token,
                    "price_hash_seed": BASE64_STANDARD.encode(reveal_bid.price_hash_seed).to_string()
                }),
            })
        }
        AuctionInstruction::CloseRequest => {
            check_num_auction_accounts(accounts, 7)?;
            Ok(ParsedInstruction {
                instruction_type: "close_request".to_string(),
                info: json!({
                    "request_authority": pubkey(&accounts[0]),
                    "job_request": pubkey(&accounts[1]),
                    "bundle_payer": pubkey(&accounts[2]),
                    "bundle": pubkey(&accounts[3]),
                    "vote_account": pubkey(&accounts[4]),
                    "vote_authority": pubkey(&accounts[5]),
                }),
            })
        }
        AuctionInstruction::AppendData => {
            check_num_auction_accounts(accounts, 3)?;
            Ok(ParsedInstruction {
                instruction_type: "append_data".to_string(),
                info: json!({
                    "data_authority": pubkey(&accounts[0]),
                    "data_account": pubkey(&accounts[1]),
                    "system_program": pubkey(&accounts[2]),
                }),
            })
        }
        AuctionInstruction::InitConfig => Err(ParseInstructionError::InstructionNotParsable),
        AuctionInstruction::OpenBundleEscrowV2 => {
            check_num_auction_accounts(accounts, 4)?;
            let open = read_auction_instruction::<OpenBundleEscrowV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "open_bundle_escrow_v2".to_string(),
                info: json!({
                    "payer": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "system_program": pubkey(&accounts[3]),
                    "bundle_version": open.bundle_version,
                    "reward_tier": request_tier(open.reward_tier),
                    "bundle_hash": BASE64_STANDARD.encode(open.bundle_hash),
                    "coordinator": pubkey(&open.coordinator),
                    "requester_refund_recipient": pubkey(&open.requester_refund_recipient),
                    "total_input_tokens": open.total_input_tokens,
                    "max_output_tokens": open.max_output_tokens,
                    "escrow_lamports": open.escrow_lamports,
                }),
            })
        }
        AuctionInstruction::CommitAuctionSettlementV2 => {
            check_num_auction_accounts(accounts, 4)?;
            let commit = read_auction_instruction::<CommitAuctionSettlementV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "commit_auction_settlement_v2".to_string(),
                info: json!({
                    "coordinator": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "winner_vote_account": pubkey(&accounts[3]),
                    "auction_hash": BASE64_STANDARD.encode(commit.auction_hash),
                    "winner_node_pubkey": pubkey(&commit.winner_node_pubkey),
                    "clearing_price_per_output_token": commit.clearing_price_per_output_token,
                }),
            })
        }
        AuctionInstruction::PostBundleResultV2 => {
            let data = &data[1..];
            let (post, input_tokens) = if data.len() == size_of::<PostBundleResultV3Args>() {
                let post = read_auction_instruction::<PostBundleResultV3Args>(data)?;
                (post.post, Some(post.input_tokens))
            } else {
                (
                    read_auction_instruction::<PostBundleResultV2Args>(data)?,
                    None,
                )
            };
            if !(3..=5).contains(&accounts.len()) {
                return Err(ParseInstructionError::InstructionKeyMismatch);
            }
            let mut info = json!({
                "authority": pubkey(&accounts[0]),
                "bundle_escrow": pubkey(&accounts[1]),
                "config_policy": pubkey(&accounts[2]),
                "bundle_verifier_page": (accounts.len() > 3)
                    .then(|| pubkey(&accounts[3])),
                "result_hash": BASE64_STANDARD.encode(post.result_hash),
                "posted_output_tokens": post.posted_output_tokens,
                "page_index": post.page_index,
                "page_entry_count": post.page_entry_count,
                "page_entries": post.page_entries
                    .iter()
                    .take(usize::from(post.page_entry_count).min(post.page_entries.len()))
                    .map(parse_page_entry)
                    .collect::<Vec<_>>(),
            });
            if let Some(input_tokens) = input_tokens {
                info["input_tokens"] = json!(input_tokens);
            }
            if accounts.len() > 4 {
                info["bundle_verification_dispute"] = json!(pubkey(&accounts[4]));
            }
            Ok(ParsedInstruction {
                instruction_type: "post_bundle_result_v2".to_string(),
                info,
            })
        }
        AuctionInstruction::FinalizeBundleVerificationV2 => {
            if accounts.len() < 6 {
                return Err(ParseInstructionError::InstructionKeyMismatch);
            }
            let finalize =
                read_auction_instruction::<FinalizeBundleVerificationV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "finalize_bundle_verification_v2".to_string(),
                info: json!({
                    "coordinator": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "winner_node": pubkey(&accounts[2]),
                    "requester_refund_recipient": pubkey(&accounts[3]),
                    "instructions_sysvar": pubkey(&accounts[4]),
                    "config_policy": pubkey(&accounts[5]),
                    "bundle_verifier_pages": accounts[6..].iter().map(pubkey)
                        .collect::<Vec<_>>(),
                    "verification_hash": BASE64_STANDARD.encode(finalize.verification_hash),
                    "accepted_output_tokens": finalize.accepted_output_tokens,
                    "winner_payout_lamports": finalize.winner_payout_lamports,
                    "verdict": verification_verdict(finalize.verdict),
                    "quorum_verifier_bitmap": finalize.quorum_verifier_bitmap,
                }),
            })
        }
        AuctionInstruction::ClaimWinnerLstakeV2 => {
            check_num_auction_accounts(accounts, 5)?;
            read_auction_instruction::<ClaimWinnerLstakeV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "claim_winner_lstake_v2".to_string(),
                info: json!({
                    "bundle_escrow": pubkey(&accounts[0]),
                    "winner_vote_account": pubkey(&accounts[1]),
                    "vote_program": pubkey(&accounts[2]),
                    "vote_authority": pubkey(&accounts[3]),
                    "config_policy": pubkey(&accounts[4]),
                }),
            })
        }
        AuctionInstruction::ClaimVerifierLstakeV2 => {
            if accounts.len() < 5 {
                return Err(ParseInstructionError::InstructionKeyMismatch);
            }
            read_auction_instruction::<ClaimVerifierLstakeV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "claim_verifier_lstake_v2".to_string(),
                info: json!({
                    "bundle_escrow": pubkey(&accounts[0]),
                    "verifier_vote_account": pubkey(&accounts[1]),
                    "vote_program": pubkey(&accounts[2]),
                    "vote_authority": pubkey(&accounts[3]),
                    "config_policy": pubkey(&accounts[4]),
                    "bundle_verifier_pages": accounts[5..].iter().map(pubkey)
                        .collect::<Vec<_>>(),
                }),
            })
        }
        AuctionInstruction::ExpireBundleEscrowV2 => {
            check_num_auction_accounts(accounts, 3)?;
            read_auction_instruction::<ExpireBundleEscrowV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "expire_bundle_escrow_v2".to_string(),
                info: json!({
                    "bundle_escrow": pubkey(&accounts[0]),
                    "requester_refund_recipient": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                }),
            })
        }
        AuctionInstruction::InitConfigPolicyV2 => {
            check_num_auction_accounts(accounts, 3)?;
            let data = &data[1..];
            let historical = data.len() == size_of::<InitConfigPolicySmallV3Args>();
            let init = if historical {
                let old = read_auction_instruction::<InitConfigPolicySmallV3Args>(data)?;
                InitConfigPolicyV2Args {
                    config_policy_lamports: old.config_policy_lamports,
                    initial_admin_authority: old.initial_admin_authority,
                    service_authority: old.service_authority,
                    policy_flags: old.policy_flags,
                    minimum_bundle_auction_pairs: old.minimum_bundle_auction_pairs,
                    max_auction_credits_per_update: old.max_auction_credits_per_update,
                    v2_verifiers_per_auction: old.v2_verifiers_per_auction,
                    v2_verifier_quorum: old.v2_verifier_quorum,
                    tier_configs: old.tier_configs,
                    ..InitConfigPolicyV2Args::zeroed()
                }
            } else {
                read_auction_instruction::<InitConfigPolicyV2Args>(data)?
            };
            let mut info = json!({
                "authority": pubkey(&accounts[0]),
                "config_policy": pubkey(&accounts[1]),
                "system_program": pubkey(&accounts[2]),
                "config_policy_lamports": init.config_policy_lamports,
                "initial_admin_authority": auction_pubkey(init.initial_admin_authority),
                "service_authority": auction_pubkey(init.service_authority),
                "policy_flags": config_policy_flags(init.policy_flags),
                "minimum_bundle_auction_pairs": init.minimum_bundle_auction_pairs,
                "max_auction_credits_per_update": init.max_auction_credits_per_update,
                "v2_verifiers_per_auction": init.v2_verifiers_per_auction,
                "v2_verifier_quorum": init.v2_verifier_quorum,
                "tier_configs": init.tier_configs.iter().map(request_tier_config).collect::<Vec<_>>(),
            });
            if !historical {
                info["missed_verification_dispute_window_slots"] =
                    json!(init.missed_verification_dispute_window_slots);
                info["dispute_verification_window_slots"] =
                    json!(init.dispute_verification_window_slots);
                info["paid_verification_dispute_window_slots"] =
                    json!(init.paid_verification_dispute_window_slots);
                info["paid_verification_dispute_bond_lamports"] =
                    json!(init.paid_verification_dispute_bond_lamports);
            }
            Ok(ParsedInstruction {
                instruction_type: "init_config_policy_v2".to_string(),
                info,
            })
        }
        AuctionInstruction::SetConfigPolicyV2 => {
            check_num_auction_accounts(accounts, 2)?;
            let data = &data[1..];
            if data.len() == size_of::<SetConfigPolicyReplayV1Args>() {
                let set = read_auction_instruction::<SetConfigPolicyReplayV1Args>(data)?;
                return Ok(ParsedInstruction {
                    instruction_type: "set_config_policy_v2".to_string(),
                    info: json!({
                        "authority": pubkey(&accounts[0]),
                        "config_policy": pubkey(&accounts[1]),
                        "patch_kind": config_policy_patch_kind(set.patch_kind),
                        "replay": replay_policy(&set.replay),
                    }),
                });
            }
            let (set, small_credit_enabled) =
                if data.len() == size_of::<SetConfigPolicySmallV3Args>() {
                    let old = read_auction_instruction::<SetConfigPolicySmallV3Args>(data)?;
                    (
                        SetConfigPolicyV2Args {
                            patch_kind: old.patch_kind,
                            authority_kind: old.authority_kind,
                            authority_index: old.authority_index,
                            v2_verifiers_per_auction: old.v2_verifiers_per_auction,
                            v2_verifier_quorum: old.v2_verifier_quorum,
                            tier: old.tier,
                            policy_flags: old.policy_flags,
                            max_auction_credits_per_update: old.max_auction_credits_per_update,
                            authority: old.authority,
                            tier_config: old.tier_config,
                            ..SetConfigPolicyV2Args::zeroed()
                        },
                        Some(old.small_credit_enabled),
                    )
                } else {
                    (
                        read_auction_instruction::<SetConfigPolicyV2Args>(data)?,
                        None,
                    )
                };
            let mut info = json!({
                "authority": pubkey(&accounts[0]),
                "config_policy": pubkey(&accounts[1]),
                "patch_kind": config_policy_patch_kind(set.patch_kind),
                "authority_kind": config_policy_authority_kind(set.authority_kind),
                "authority_index": set.authority_index,
                "v2_verifiers_per_auction": set.v2_verifiers_per_auction,
                "v2_verifier_quorum": set.v2_verifier_quorum,
                "tier": request_tier(set.tier),
                "policy_flags": config_policy_flags(set.policy_flags),
                "max_auction_credits_per_update": set.max_auction_credits_per_update,
                "authority_pubkey": auction_pubkey(set.authority),
                "tier_config": request_tier_config(&set.tier_config),
            });
            if let Some(enabled) = small_credit_enabled {
                if set.patch_kind == ConfigPolicyV2PatchKind::SMALL_CREDIT_SETTINGS {
                    info["small_credit_enabled"] = json!(enabled);
                }
            } else {
                info["missed_verification_dispute_window_slots"] =
                    json!(set.missed_verification_dispute_window_slots);
                info["dispute_verification_window_slots"] =
                    json!(set.dispute_verification_window_slots);
                info["paid_verification_dispute_window_slots"] =
                    json!(set.paid_verification_dispute_window_slots);
                info["paid_verification_dispute_bond_lamports"] =
                    json!(set.paid_verification_dispute_bond_lamports);
            }
            Ok(ParsedInstruction {
                instruction_type: "set_config_policy_v2".to_string(),
                info,
            })
        }
        AuctionInstruction::InitBundleVerifierPageV2 => {
            check_num_auction_accounts(accounts, 4)?;
            let init = read_auction_instruction::<InitBundleVerifierPageV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "init_bundle_verifier_page_v2".to_string(),
                info: json!({
                    "payer": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "bundle_verifier_page": pubkey(&accounts[2]),
                    "system_program": pubkey(&accounts[3]),
                    "bundle_verifier_page_lamports": init.bundle_verifier_page_lamports,
                    "page_index": init.page_index,
                }),
            })
        }
        AuctionInstruction::DisputeBundleVerificationV2 => {
            check_num_auction_accounts(accounts, 6)?;
            let dispute = read_auction_instruction::<DisputeBundleVerificationV2Args>(&data[1..])?;
            let kind = match dispute.kind {
                BundleVerificationDisputeV2Kind::Unset => "unset",
                BundleVerificationDisputeV2Kind::MissedVerification => "missed_verification",
                BundleVerificationDisputeV2Kind::PaidVerdictDispute => "paid_verdict_dispute",
                _ => "unknown",
            };
            Ok(ParsedInstruction {
                instruction_type: "dispute_bundle_verification_v2".to_string(),
                info: json!({
                    "dispute_payer": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "bundle_verification_dispute": pubkey(&accounts[2]),
                    "bond_refund_recipient": pubkey(&accounts[3]),
                    "config_policy": pubkey(&accounts[4]),
                    "system_program": pubkey(&accounts[5]),
                    "kind": {"name": kind, "value": u8::from(dispute.kind)},
                }),
            })
        }
        AuctionInstruction::SelectBundleVerifiersV2 => {
            if !(3..=4).contains(&accounts.len()) {
                return Err(ParseInstructionError::InstructionKeyMismatch);
            }
            read_auction_instruction::<SelectBundleVerifiersV2Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "select_bundle_verifiers_v2".to_string(),
                info: json!({
                    "bundle_escrow": pubkey(&accounts[0]),
                    "auction_verifiers": pubkey(&accounts[1]),
                    "slot_hashes": pubkey(&accounts[2]),
                    "bundle_verification_dispute": (accounts.len() > 3)
                        .then(|| pubkey(&accounts[3])),
                }),
            })
        }
        AuctionInstruction::SlashSmallCredits => {
            check_num_auction_accounts(accounts, 5)?;
            let slash = read_auction_instruction::<SlashSmallCreditsArgs>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "slash_small_credits".to_string(),
                info: json!({
                    "slash_authority": pubkey(&accounts[0]),
                    "config_policy": pubkey(&accounts[1]),
                    "mint": pubkey(&accounts[2]),
                    "token_account": pubkey(&accounts[3]),
                    "token_program": pubkey(&accounts[4]),
                    "amount": slash.amount,
                    "sequence": slash.sequence,
                }),
            })
        }
        AuctionInstruction::OpenBundleEscrowV4 => {
            check_num_auction_accounts(accounts, 4)?;
            let open = read_auction_instruction::<OpenBundleEscrowV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "open_bundle_escrow_v4".to_string(),
                info: json!({
                    "payer": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "system_program": pubkey(&accounts[3]),
                    "bundle_version": open.bundle_version,
                    "reward_tier": request_tier(open.reward_tier),
                    "bundle_hash": BASE64_STANDARD.encode(open.bundle_hash),
                    "coordinator": pubkey(&open.coordinator),
                    "requester_refund_recipient": pubkey(&open.requester_refund_recipient),
                    "total_input_tokens": open.total_input_tokens,
                    "max_output_tokens": open.max_output_tokens,
                    "escrow_lamports": open.escrow_lamports,
                    "expected_page_count": open.expected_page_count,
                }),
            })
        }
        AuctionInstruction::CloseBundleVerifierPageV4 => {
            check_num_auction_accounts(accounts, 3)?;
            let close = read_auction_instruction::<CloseBundleVerifierPageV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "close_bundle_verifier_page_v4".to_string(),
                info: json!({
                    "funder": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "bundle_verifier_page": pubkey(&accounts[2]),
                    "page_index": close.page_index,
                    "disputed": close.disputed,
                }),
            })
        }
        AuctionInstruction::AuthorizeBundleDisputeEvidenceV4 => {
            check_num_auction_accounts(accounts, 4)?;
            let authorize =
                read_auction_instruction::<AuthorizeBundleDisputeEvidenceV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "authorize_bundle_dispute_evidence_v4".to_string(),
                info: json!({
                    "submitter": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "bundle_verification_dispute": pubkey(&accounts[2]),
                    "instructions_sysvar": pubkey(&accounts[3]),
                    "verification_hash": BASE64_STANDARD.encode(authorize.verification_hash),
                    "page_hashes": authorize.page_hashes.iter().map(|hash| BASE64_STANDARD.encode(hash)).collect::<Vec<_>>(),
                    "quorum_verifier_bitmap": authorize.quorum_verifier_bitmap,
                }),
            })
        }
        AuctionInstruction::ClaimSmallCreditsV4 => {
            check_num_auction_accounts(accounts, 5)?;
            read_auction_instruction::<ClaimSmallCreditsV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "claim_small_credits_v4".to_string(),
                info: json!({
                    "bundle_escrow": pubkey(&accounts[0]),
                    "config_policy": pubkey(&accounts[1]),
                    "mint": pubkey(&accounts[2]),
                    "token_account": pubkey(&accounts[3]),
                    "token_program": pubkey(&accounts[4]),
                }),
            })
        }
        AuctionInstruction::OpenPricedBundleEscrowV4 => {
            check_num_auction_accounts(accounts, 4)?;
            let open = read_auction_instruction::<OpenPricedBundleEscrowV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "open_priced_bundle_escrow_v4".to_string(),
                info: json!({
                    "payer": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "system_program": pubkey(&accounts[3]),
                    "bundle_version": open.bundle_version,
                    "reward_tier": request_tier(open.reward_tier),
                    "bundle_hash": BASE64_STANDARD.encode(open.bundle_hash),
                    "coordinator": pubkey(&open.coordinator),
                    "requester_refund_recipient": pubkey(&open.requester_refund_recipient),
                    "total_input_tokens": open.total_input_tokens,
                    "max_output_tokens": open.max_output_tokens,
                    "escrow_lamports": open.escrow_lamports,
                    "expected_page_count": open.expected_page_count,
                    "pricing_commitment": BASE64_STANDARD.encode(open.pricing_commitment),
                }),
            })
        }
        AuctionInstruction::PostBundlePricing => {
            check_num_auction_accounts(accounts, 3)?;
            let post = read_auction_instruction::<PostBundlePricingArgs>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "post_bundle_pricing".to_string(),
                info: json!({
                    "coordinator": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "bundle_verifier_page": pubkey(&accounts[2]),
                    "page_index": post.page_index,
                    "pricing_entry_count": post.pricing_entry_count,
                    "pricing_entries": post.pricing_entries.iter().take(usize::from(post.pricing_entry_count)).map(|entry| json!({
                        "job_id": auction_pubkey(entry.job_id),
                        "max_output_tokens": entry.max_output_tokens,
                        "price_per_output_token": entry.price_per_output_token,
                    })).collect::<Vec<_>>(),
                }),
            })
        }
        AuctionInstruction::SealBundlePricing => {
            check_num_auction_accounts(accounts, 2)?;
            read_auction_instruction::<SealBundlePricingArgs>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "seal_bundle_pricing".to_string(),
                info: json!({
                    "coordinator": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "bundle_verifier_pages": accounts[2..].iter().map(pubkey).collect::<Vec<_>>(),
                }),
            })
        }
        AuctionInstruction::CommitAuctionSettlementV3 => {
            check_num_auction_accounts(accounts, 4)?;
            let commit = read_auction_instruction::<CommitAuctionSettlementV3Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "commit_auction_settlement_v3".to_string(),
                info: json!({
                    "coordinator": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "winner_vote_account": pubkey(&accounts[3]),
                    "auction_hash": BASE64_STANDARD.encode(commit.auction_hash),
                    "winner_node_pubkey": pubkey(&commit.winner_node_pubkey),
                }),
            })
        }
        AuctionInstruction::OpenSmallBundleEscrowV4 => {
            check_num_auction_accounts(accounts, 4)?;
            let open = read_auction_instruction::<OpenSmallBundleEscrowV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "open_small_bundle_escrow_v4".to_string(),
                info: json!({
                    "payer": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "system_program": pubkey(&accounts[3]),
                    "bundle_version": open.bundle_version,
                    "bundle_hash": BASE64_STANDARD.encode(open.bundle_hash),
                    "coordinator": pubkey(&open.coordinator),
                    "requester_refund_recipient": pubkey(&open.requester_refund_recipient),
                    "total_input_tokens": open.total_input_tokens,
                    "max_output_tokens": open.max_output_tokens,
                    "expected_page_count": open.expected_page_count,
                    "winner_node_pubkey": pubkey(&open.winner_node_pubkey),
                }),
            })
        }
        AuctionInstruction::CommitSmallAuctionSettlementV4 => {
            check_num_auction_accounts(accounts, 3)?;
            let commit =
                read_auction_instruction::<CommitSmallAuctionSettlementV4Args>(&data[1..])?;
            Ok(ParsedInstruction {
                instruction_type: "commit_small_auction_settlement_v4".to_string(),
                info: json!({
                    "coordinator": pubkey(&accounts[0]),
                    "bundle_escrow": pubkey(&accounts[1]),
                    "config_policy": pubkey(&accounts[2]),
                    "auction_hash": BASE64_STANDARD.encode(commit.auction_hash),
                }),
            })
        }
    }
}

fn check_num_auction_accounts(
    accounts: &[[u8; 32]],
    num: usize,
) -> Result<(), ParseInstructionError> {
    if accounts.len() < num {
        Err(ParseInstructionError::InstructionKeyMismatch)
    } else {
        Ok(())
    }
}

fn read_auction_instruction<T: bytemuck::Pod>(data: &[u8]) -> Result<T, ParseInstructionError> {
    try_pod_read_unaligned::<T>(data).map_err(|_| ParseInstructionError::InstructionNotParsable)
}

fn pubkey(pubkey: &[u8; 32]) -> String {
    bs58::encode(pubkey).into_string()
}

fn auction_pubkey(pubkey: crate::Pubkey) -> String {
    bs58::encode(pubkey.as_ref()).into_string()
}

fn parse_page_entry(entry: &crate::BundleVerifierPageV2Entry) -> Value {
    json!({
        "job_id": auction_pubkey(entry.job_id),
        "posted_output_tokens": entry.posted_output_tokens,
        "accepted_output_tokens": entry.accepted_output_tokens,
        "assigned_verifiers_token_ranges": entry.assigned_verifiers_token_ranges,
        "verifier_reward_tokens": entry.verifier_reward_tokens,
        "verdict": verification_verdict(entry.verdict),
        "verifier_claimed_bitmap": entry.verifier_claimed_bitmap,
    })
}

fn verification_verdict(verdict: VerificationVerdictV2) -> Value {
    let value = u8::from(verdict);
    let name = match verdict {
        VerificationVerdictV2::Unset => "unset",
        VerificationVerdictV2::Verified => "verified",
        VerificationVerdictV2::Rejected => "rejected",
        _ => "unknown",
    };
    json!({
        "name": name,
        "value": value,
    })
}

fn request_tier(value: u64) -> Value {
    match RequestTier::try_from(value) {
        Ok(RequestTier::Eco) => json!({"name": "eco", "value": value}),
        Ok(RequestTier::Small) => json!({"name": "small", "value": value}),
        Ok(RequestTier::Standard) => json!({"name": "standard", "value": value}),
        Ok(RequestTier::Pro) => json!({"name": "pro", "value": value}),
        Ok(RequestTier::Large) => json!({"name": "large", "value": value}),
        Err(_) => json!({"name": "unknown", "value": value}),
    }
}

fn request_tier_config(config: &RequestTierConfigV2) -> Value {
    json!({
        "bid_reveal_duration": config.bid_reveal_duration,
        "active_auction_duration": config.active_auction_duration,
        "bundle_duration": config.bundle_duration,
        "requests_per_bundle": config.requests_per_bundle,
        "max_context_length_tokens": config.max_context_length_tokens,
        "job_submission_duration_slots": config.job_submission_duration_slots,
        "bid_commitment_amount_multiplier": config.bid_commitment_amount_multiplier,
        "auction_credits_multiplier": config.auction_credits_multiplier,
        "settlement_window_slots": config.settlement_window_slots,
        "result_window_slots": config.result_window_slots,
        "verification_window_slots": config.verification_window_slots,
        "claim_window_slots": config.claim_window_slots,
    })
}

fn config_policy_flags(flags: ConfigPolicyV2Flags) -> Value {
    let names = [
        (
            ConfigPolicyV2Flag::AllowServiceOpenEscrowArgsBypass,
            "allow_service_open_escrow_args_bypass",
        ),
        (
            ConfigPolicyV2Flag::AllowServiceCommitOverride,
            "allow_service_commit_override",
        ),
        (
            ConfigPolicyV2Flag::AllowServiceResultPostOverride,
            "allow_service_result_post_override",
        ),
        (
            ConfigPolicyV2Flag::AllowServiceFinalizeOverride,
            "allow_service_finalize_override",
        ),
        (
            ConfigPolicyV2Flag::AllowServicePageBackedFinalizeBypass,
            "allow_service_page_backed_finalize_bypass",
        ),
        (
            ConfigPolicyV2Flag::AllowServicePageBackedFinalizePayout,
            "allow_service_page_backed_finalize_payout",
        ),
    ]
    .into_iter()
    .filter_map(|(flag, name)| flags.contains(flag).then_some(name))
    .collect::<Vec<_>>();

    json!({
        "bits": flags.bits(),
        "names": names,
    })
}

fn config_policy_patch_kind(kind: ConfigPolicyV2PatchKind) -> Value {
    let name = if kind == ConfigPolicyV2PatchKind::FLAGS {
        "flags"
    } else if kind == ConfigPolicyV2PatchKind::AUTHORITY {
        "authority"
    } else if kind == ConfigPolicyV2PatchKind::VERIFIER_SETTINGS {
        "verifier_settings"
    } else if kind == ConfigPolicyV2PatchKind::TIER_CONFIG {
        "tier_config"
    } else if kind == ConfigPolicyV2PatchKind::MAX_AUCTION_CREDITS_PER_UPDATE {
        "max_auction_credits_per_update"
    } else if kind == ConfigPolicyV2PatchKind::DISPUTE_SETTINGS {
        "dispute_settings"
    } else if kind == ConfigPolicyV2PatchKind::SMALL_CREDIT_SLASH_AUTHORITY {
        "small_credit_slash_authority"
    } else if kind == ConfigPolicyV2PatchKind::SMALL_CREDIT_SETTINGS {
        "small_credit_settings"
    } else if kind == ConfigPolicyV2PatchKind::REPLAY_SETTINGS {
        "replay_settings"
    } else {
        "unknown"
    };
    json!({
        "name": name,
        "value": kind.0,
    })
}

fn replay_policy(policy: &ReplayPolicyV1) -> Value {
    json!({
        "class_id": BASE64_STANDARD.encode(policy.class_id),
        "version": policy.version,
        "tier_mask": policy.tier_mask,
        "arbiters": policy.arbiters,
        "hold_slots": policy.hold_slots,
        "forfeit_mask": policy.forfeit_mask,
        "suspension_hours": policy.suspension_hours.to_vec(),
    })
}

fn config_policy_authority_kind(kind: ConfigPolicyV2AuthorityKind) -> Value {
    let name = if kind == ConfigPolicyV2AuthorityKind::ADMIN {
        "admin"
    } else if kind == ConfigPolicyV2AuthorityKind::SERVICE {
        "service"
    } else {
        "unknown"
    };
    json!({
        "name": name,
        "value": kind.0,
    })
}
