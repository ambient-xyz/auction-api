//! Shank extraction surface for the production V2 wire interface.
//!
//! These types describe the existing POD ABI; the program does not use them at
//! runtime. Keep them behind the `idl` feature and verify every mirror against
//! the production types in `tests/v2_idl_conformance.rs`.

#![allow(dead_code)]

use shank::{ShankAccount, ShankInstruction, ShankType};

type Pubkey = [u8; 32];

#[derive(ShankInstruction)]
#[repr(u8)]
pub enum AuctionV2Instruction {
    #[account(0, name = "payer", mut, sig)]
    #[account(1, name = "bundle_escrow", mut)]
    #[account(2, name = "config_policy", mut)]
    #[account(3, name = "system_program")]
    OpenBundleEscrowV2(OpenBundleEscrowV2Args) = 12,

    #[account(0, name = "coordinator", mut, sig)]
    #[account(1, name = "bundle_escrow", mut)]
    #[account(2, name = "config_policy", mut)]
    #[account(3, name = "winner_vote_account", mut)]
    CommitAuctionSettlementV2(CommitAuctionSettlementV2Args) = 13,

    #[account(0, name = "authority", mut, sig)]
    #[account(1, name = "bundle_escrow", mut)]
    #[account(2, name = "config_policy", mut)]
    #[account(3, name = "bundle_verifier_page", mut, optional)]
    PostBundleResultV2(PostBundleResultV2Args) = 14,

    /// Page and dispute accounts follow this fixed prefix. See `idl/V2_INTERFACE.md`.
    #[account(0, name = "coordinator", mut, sig)]
    #[account(1, name = "bundle_escrow", mut)]
    #[account(2, name = "winner_node", mut)]
    #[account(3, name = "requester_refund_recipient", mut)]
    #[account(4, name = "instructions_sysvar")]
    #[account(5, name = "config_policy", mut)]
    FinalizeBundleVerificationV2(FinalizeBundleVerificationV2Args) = 15,

    #[account(0, name = "bundle_escrow", mut)]
    #[account(1, name = "winner_vote_account", mut)]
    #[account(2, name = "vote_program")]
    #[account(3, name = "vote_authority", mut, sig)]
    #[account(4, name = "config_policy", mut)]
    ClaimWinnerLstakeV2 = 16,

    /// Verifier page accounts follow this fixed prefix. See `idl/V2_INTERFACE.md`.
    #[account(0, name = "bundle_escrow", mut)]
    #[account(1, name = "verifier_vote_account", mut)]
    #[account(2, name = "vote_program")]
    #[account(3, name = "vote_authority", mut, sig)]
    #[account(4, name = "config_policy", mut)]
    ClaimVerifierLstakeV2 = 17,

    /// Dispute settlement accounts may follow this prefix. See `idl/V2_INTERFACE.md`.
    #[account(0, name = "bundle_escrow", mut)]
    #[account(1, name = "requester_refund_recipient", mut)]
    #[account(2, name = "config_policy", mut)]
    ExpireBundleEscrowV2 = 18,

    #[account(0, name = "authority", mut, sig)]
    #[account(1, name = "config_policy", mut)]
    #[account(2, name = "system_program")]
    InitConfigPolicyV2(InitConfigPolicyV2Args) = 19,

    #[account(0, name = "authority", mut, sig)]
    #[account(1, name = "config_policy", mut)]
    SetConfigPolicyV2(SetConfigPolicyV2Args) = 20,

    #[account(0, name = "payer", mut, sig)]
    #[account(1, name = "bundle_escrow")]
    #[account(2, name = "bundle_verifier_page", mut)]
    #[account(3, name = "system_program")]
    InitBundleVerifierPageV2(InitBundleVerifierPageV2Args) = 21,

    #[account(0, name = "dispute_payer", mut, sig)]
    #[account(1, name = "bundle_escrow", mut)]
    #[account(2, name = "bundle_verification_dispute", mut)]
    #[account(3, name = "bond_refund_recipient", mut)]
    #[account(4, name = "config_policy", mut)]
    #[account(5, name = "system_program")]
    DisputeBundleVerificationV2(DisputeBundleVerificationV2Args) = 22,

    #[account(0, name = "bundle_escrow", mut)]
    #[account(1, name = "bundle_verification_dispute", mut, optional)]
    SelectBundleVerifiersV2 = 23,
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct OpenBundleEscrowV2Args {
    pub bundle_version: u32,
    #[padding]
    pub reserved0: [u8; 4],
    pub reward_tier: u64,
    pub bundle_hash: [u8; 32],
    pub coordinator: Pubkey,
    pub requester_refund_recipient: Pubkey,
    pub total_input_tokens: u64,
    pub max_output_tokens: u64,
    pub escrow_lamports: u64,
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct CommitAuctionSettlementV2Args {
    pub auction_hash: [u8; 32],
    pub winner_node_pubkey: Pubkey,
    pub clearing_price_per_output_token: u64,
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct PostBundleResultV2Args {
    pub result_hash: [u8; 32],
    pub posted_output_tokens: u64,
    pub page_index: u16,
    pub page_entry_count: u16,
    #[padding]
    pub reserved: [u8; 4],
    pub page_entries: [BundleVerifierPageV2Entry; 6],
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct FinalizeBundleVerificationV2Args {
    pub verification_hash: [u8; 32],
    pub accepted_output_tokens: u64,
    pub winner_payout_lamports: u64,
    /// `0 = Unset`, `1 = Verified`, `2 = Rejected`.
    pub verdict: u8,
    pub quorum_verifier_bitmap: u8,
    #[padding]
    pub reserved: [u8; 6],
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct InitConfigPolicyV2Args {
    pub config_policy_lamports: u64,
    pub initial_admin_authority: Pubkey,
    pub service_authority: Pubkey,
    pub policy_flags: u64,
    pub minimum_bundle_auction_pairs: u64,
    pub max_auction_credits_per_update: u64,
    pub v2_verifiers_per_auction: u8,
    pub v2_verifier_quorum: u8,
    #[padding]
    pub reserved0: [u8; 6],
    pub missed_verification_dispute_window_slots: u64,
    pub dispute_verification_window_slots: u64,
    pub paid_verification_dispute_window_slots: u64,
    pub paid_verification_dispute_bond_lamports: u64,
    pub tier_configs: [RequestTierConfigV2; 5],
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct SetConfigPolicyV2Args {
    pub patch_kind: u8,
    pub authority_kind: u8,
    pub authority_index: u8,
    pub v2_verifiers_per_auction: u8,
    pub v2_verifier_quorum: u8,
    #[padding]
    pub reserved0: [u8; 3],
    pub tier: u64,
    pub policy_flags: u64,
    pub max_auction_credits_per_update: u64,
    pub missed_verification_dispute_window_slots: u64,
    pub dispute_verification_window_slots: u64,
    pub paid_verification_dispute_window_slots: u64,
    pub paid_verification_dispute_bond_lamports: u64,
    pub authority: Pubkey,
    pub tier_config: RequestTierConfigV2,
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct InitBundleVerifierPageV2Args {
    pub bundle_verifier_page_lamports: u64,
    pub page_index: u16,
    #[padding]
    pub reserved: [u8; 6],
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct DisputeBundleVerificationV2Args {
    /// `0 = Unset`, `1 = MissedVerification`, `2 = PaidVerdictDispute`.
    pub kind: u8,
    #[padding]
    pub reserved: [u8; 7],
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct RequestTierConfigV2 {
    pub bid_reveal_duration: u64,
    pub active_auction_duration: u64,
    pub bundle_duration: u64,
    pub requests_per_bundle: u64,
    pub max_context_length_tokens: u64,
    pub job_submission_duration_slots: u64,
    pub bid_commitment_amount_multiplier: u64,
    pub auction_credits_multiplier: u64,
    pub settlement_window_slots: u64,
    pub result_window_slots: u64,
    pub verification_window_slots: u64,
    pub claim_window_slots: u64,
}

#[derive(ShankType)]
#[repr(C)]
pub struct AccountHeaderV1 {
    pub discriminator: u8,
    pub version: u8,
    #[padding]
    pub reserved: [u8; 6],
}

#[derive(borsh::BorshSerialize, ShankType)]
#[repr(C)]
pub struct BundleVerifierPageV2Entry {
    pub job_id: Pubkey,
    pub posted_output_tokens: u64,
    pub accepted_output_tokens: u64,
    pub assigned_verifiers_token_ranges: [u64; 6],
    pub verifier_reward_tokens: [u64; 3],
    /// `0 = Unset`, `1 = Verified`, `2 = Rejected`.
    pub verdict: u8,
    pub verifier_claimed_bitmap: u8,
    #[padding]
    pub reserved: [u8; 6],
}

#[derive(ShankType)]
#[repr(C)]
pub struct RawBundleEscrowV2Data {
    /// See `BUNDLE_ESCROW_V2_STATUS_*` constants.
    pub status: u64,
    pub reward_tier: u64,
    pub coordinator: Pubkey,
    pub requester_refund_recipient: Pubkey,
    pub bundle_version: u32,
    #[padding]
    pub reserved0: [u8; 4],
    pub bundle_hash: [u8; 32],
    pub total_input_tokens: u64,
    pub max_output_tokens: u64,
    pub escrow_lamports: u64,
    pub winner_node_pubkey: Pubkey,
    pub winner_vote_account: Pubkey,
    pub clearing_price_per_output_token: u64,
    pub selected_verifiers: [Pubkey; 3],
    pub auction_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub verification_hash: [u8; 32],
    pub posted_output_tokens: u64,
    pub accepted_output_tokens: u64,
    pub winner_payout_lamports: u64,
    pub settlement_deadline_slot: u64,
    pub result_deadline_slot: u64,
    pub verification_deadline_slot: u64,
    pub claim_deadline_slot: u64,
    pub winner_reward_claimed: u8,
    pub verifier_reward_claimed_bitmap: u8,
    pub quorum_verifier_bitmap: u8,
    pub verifier_page_count: u8,
    #[padding]
    pub reserved1: [u8; 4],
    pub verifier_reward_remaining: [u64; 3],
}

#[derive(ShankType)]
#[repr(C)]
pub struct BundleEscrowV2ReservedData {
    pub provisional_challenge_deadline_slot: u64,
    pub verifier_selection_slot: u64,
    pub verifier_selection_epoch: u64,
    pub paid_verification_dispute_bond_lamports: u64,
    pub winner_auction_credits: u64,
    pub max_auction_credits_per_update: u64,
    pub missed_verification_dispute_window_slots: u32,
    pub replacement_verification_window_slots: u32,
    pub paid_verification_dispute_window_slots: u32,
    pub verifier_selection_phase: u8,
    pub verifier_selection_pending: u8,
    pub verifier_count: u8,
    pub verifier_quorum: u8,
}

/// PDA seeds: `bundle_escrow_v2`, payer, bundle hash, little-endian bundle version.
#[derive(ShankAccount)]
#[repr(C)]
pub struct BundleEscrowV2 {
    pub header: AccountHeaderV1,
    pub data: RawBundleEscrowV2Data,
    pub reserved: BundleEscrowV2ReservedData,
}

#[derive(ShankType)]
#[repr(C)]
pub struct RawBundleVerifierPageV2Data {
    pub bundle_escrow: Pubkey,
    pub page_index: u16,
    pub entry_count: u16,
    #[padding]
    pub reserved0: [u8; 4],
    pub entries: [BundleVerifierPageV2Entry; 6],
}

/// Canonical PDA seeds: `bundle_verifier_page_v2`, escrow, little-endian page index.
/// Dispute staging pages replace the literal with `bundle_dispute_verifier_page_v2`.
#[derive(ShankAccount)]
#[repr(C)]
pub struct BundleVerifierPageV2 {
    pub header: AccountHeaderV1,
    pub data: RawBundleVerifierPageV2Data,
    #[padding]
    pub reserved: [u8; 64],
}

#[derive(ShankType)]
#[repr(C)]
pub struct RawBundleVerificationDisputeV2Data {
    pub bundle_escrow: Pubkey,
    /// See `BUNDLE_VERIFICATION_DISPUTE_V2_KIND_*` constants.
    pub kind: u8,
    /// See `VERIFICATION_VERDICT_V2_*` constants.
    pub original_verdict: u8,
    pub bump: u8,
    pub replacement_verifier_count: u8,
    pub replacement_verifier_quorum: u8,
    #[padding]
    pub reserved0: [u8; 3],
    pub replacement_verifiers: [Pubkey; 3],
    pub replacement_deadline_slot: u64,
    pub bond_lamports: u64,
    pub bond_refund_recipient: Pubkey,
}

/// PDA seeds: `bundle_verification_dispute_v2`, bundle escrow.
#[derive(ShankAccount)]
#[repr(C)]
pub struct BundleVerificationDisputeV2 {
    pub header: AccountHeaderV1,
    pub data: RawBundleVerificationDisputeV2Data,
}

/// PDA seeds: `global_config`, `policy_v2`.
#[derive(ShankAccount)]
#[repr(C)]
pub struct ConfigPolicyV2 {
    pub bump: u64,
    pub minimum_bundle_auction_pairs: u64,
    pub policy_flags: u64,
    pub max_auction_credits_per_update: u64,
    pub admin_authorities: [Pubkey; 8],
    pub service_authorities: [Pubkey; 16],
    pub v2_verifiers_per_auction: u8,
    pub v2_verifier_quorum: u8,
    #[padding]
    pub reserved1: [u8; 6],
    pub tier_configs: [RequestTierConfigV2; 5],
    pub missed_verification_dispute_window_slots: u64,
    pub dispute_verification_window_slots: u64,
    pub paid_verification_dispute_window_slots: u64,
    pub paid_verification_dispute_bond_lamports: u64,
    pub reserved_words: [ReservedWordV2; 7],
    pub v2_account_layout_version: u8,
    #[padding]
    pub reserved2: [u8; 7],
    #[padding]
    pub reserved_tail: [u8; 16],
}

#[derive(ShankType)]
#[repr(C)]
pub struct ReservedWordV2 {
    pub bytes: [u8; 32],
}

/// Constants copied into the generated IDLs by `tools/idl/generate.mjs`.
pub mod constants {
    pub const PROGRAM_ID: &str = "Auction111111111111111111111111111111111111";
    pub const CONFIG_POLICY_V2_SEED: &str = "policy_v2";
    pub const BUNDLE_ESCROW_V2_SEED: &str = "bundle_escrow_v2";
    pub const BUNDLE_VERIFIER_PAGE_V2_SEED: &str = "bundle_verifier_page_v2";
    pub const BUNDLE_DISPUTE_VERIFIER_PAGE_V2_SEED: &str = "bundle_dispute_verifier_page_v2";
    pub const BUNDLE_VERIFICATION_DISPUTE_V2_SEED: &str = "bundle_verification_dispute_v2";
    pub const MAX_VERIFIERS_PER_AUCTION: u64 = 3;
    pub const MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES: u64 = 6;
}
