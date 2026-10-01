use ambient_auction_api::{
    bundle_account_len, AccountLayoutVersion, AuctionInstruction, BundleJobPricingV6,
    BundlePricingCommitmentV6Message, CommitAuctionSettlementV2Args,
    CommitAuctionSettlementV3Accounts, CommitAuctionSettlementV3Args, InstructionAccounts,
    InstructionBytes, OpenBundleEscrowV5Args, OpenBundleEscrowV6Args, PostBundlePricingAccounts,
    PostBundlePricingArgs, SealBundlePricingAccounts, SealBundlePricingArgs,
    BUNDLE_PRICING_V6_DOMAIN, MAX_BUNDLE_JOBS, MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES,
};
use bytemuck::Zeroable;
use memoffset::offset_of;
use std::mem::size_of;

fn pricing(
    job_byte: u8,
    max_output_tokens: u64,
    price_per_output_token: u64,
) -> BundleJobPricingV6 {
    BundleJobPricingV6 {
        job_id: [job_byte; 32].into(),
        max_output_tokens,
        price_per_output_token,
    }
}

#[test]
fn pricing_commitment_has_a_fixed_canonical_layout_and_zero_padding() {
    let entries = [pricing(1, 100, 7), pricing(2, 200, 11)];
    let message = BundlePricingCommitmentV6Message::new([9; 32], &entries).unwrap();

    assert_eq!(size_of::<BundlePricingCommitmentV6Message>(), 936);
    assert_eq!(message.as_bytes().len(), 936);
    assert_eq!(message.domain, BUNDLE_PRICING_V6_DOMAIN);
    assert_eq!(&message.domain[..25], b"ambient.bundle.pricing.v6");
    assert_eq!(&message.domain[25..], &[0; 7]);
    assert_eq!(message.bundle_hash, [9; 32]);
    assert_eq!(message.pricing_entry_count, 2);
    assert_eq!(message._reserved0, [0; 7]);
    assert_eq!(&message.pricing_entries[..2], &entries);
    assert_eq!(
        message.pricing_entries[2..],
        [BundleJobPricingV6::default(); MAX_BUNDLE_JOBS - 2]
    );

    // These offsets define the exact bytes all hash producers must agree on.
    let bytes = message.as_bytes();
    assert_eq!(&bytes[0..32], &BUNDLE_PRICING_V6_DOMAIN);
    assert_eq!(&bytes[32..64], &[9; 32]);
    assert_eq!(bytes[64], 2);
    assert_eq!(&bytes[65..72], &[0; 7]);
    assert_eq!(&bytes[72..104], &[1; 32]);
    assert_eq!(&bytes[104..112], &100u64.to_le_bytes());
    assert_eq!(&bytes[112..120], &7u64.to_le_bytes());
}

#[test]
fn pricing_commitment_rejects_invalid_counts_and_binds_every_pricing_field() {
    assert!(BundlePricingCommitmentV6Message::new([1; 32], &[]).is_none());
    let maximum_entries = [BundleJobPricingV6::default(); MAX_BUNDLE_JOBS];
    let maximum_message = BundlePricingCommitmentV6Message::new([1; 32], &maximum_entries).unwrap();
    assert_eq!(
        usize::from(maximum_message.pricing_entry_count),
        MAX_BUNDLE_JOBS
    );
    assert!(BundlePricingCommitmentV6Message::new(
        [1; 32],
        &[BundleJobPricingV6::default(); MAX_BUNDLE_JOBS + 1],
    )
    .is_none());

    let entries = [pricing(1, 100, 7), pricing(2, 200, 11)];
    let baseline = BundlePricingCommitmentV6Message::new([9; 32], &entries)
        .unwrap()
        .as_bytes()
        .to_vec();

    let mut changed = entries;
    changed[0].job_id = [3; 32].into();
    assert_ne!(
        BundlePricingCommitmentV6Message::new([9; 32], &changed)
            .unwrap()
            .as_bytes(),
        baseline
    );
    changed = entries;
    changed[0].max_output_tokens += 1;
    assert_ne!(
        BundlePricingCommitmentV6Message::new([9; 32], &changed)
            .unwrap()
            .as_bytes(),
        baseline
    );
    changed = entries;
    changed[0].price_per_output_token += 1;
    assert_ne!(
        BundlePricingCommitmentV6Message::new([9; 32], &changed)
            .unwrap()
            .as_bytes(),
        baseline
    );
    changed.swap(0, 1);
    assert_ne!(
        BundlePricingCommitmentV6Message::new([9; 32], &changed)
            .unwrap()
            .as_bytes(),
        baseline
    );
    assert_ne!(
        BundlePricingCommitmentV6Message::new([8; 32], &entries)
            .unwrap()
            .as_bytes(),
        baseline
    );
}

#[test]
fn v6_instruction_discriminators_sizes_and_round_trips_are_stable() {
    let mut open = OpenBundleEscrowV6Args::zeroed();
    open.bundle_version = 17;
    open.expected_page_count = 3;
    open.pricing_commitment = [4; 32];
    let open_bytes = open.to_bytes();
    assert_eq!(size_of::<OpenBundleEscrowV6Args>(), 176);
    assert_eq!(offset_of!(OpenBundleEscrowV6Args, expected_page_count), 136);
    assert_eq!(offset_of!(OpenBundleEscrowV6Args, pricing_commitment), 144);
    assert_eq!(open_bytes.len(), 177);
    assert_eq!(open_bytes[0], 30);
    assert_eq!(open_bytes[0], AuctionInstruction::OpenBundleEscrowV6 as u8);
    assert_eq!(
        OpenBundleEscrowV6Args::try_from(&open_bytes[1..]).unwrap(),
        open
    );
    assert!(OpenBundleEscrowV5Args::try_from(&open_bytes[1..]).is_err());

    let mut post = PostBundlePricingArgs::zeroed();
    post.page_index = 2;
    post.pricing_entry_count = 1;
    post.pricing_entries[0] = pricing(5, 300, 13);
    let post_bytes = post.to_bytes();
    assert_eq!(size_of::<PostBundlePricingArgs>(), 296);
    assert_eq!(offset_of!(PostBundlePricingArgs, page_index), 0);
    assert_eq!(offset_of!(PostBundlePricingArgs, pricing_entry_count), 2);
    assert_eq!(offset_of!(PostBundlePricingArgs, pricing_entries), 8);
    assert_eq!(post_bytes.len(), 297);
    assert_eq!(post_bytes[0], 31);
    assert_eq!(post_bytes[0], AuctionInstruction::PostBundlePricing as u8);
    assert_eq!(
        PostBundlePricingArgs::try_from(&post_bytes[1..]).unwrap(),
        post
    );

    let seal = SealBundlePricingArgs::zeroed();
    let seal_bytes = seal.to_bytes();
    assert_eq!(size_of::<SealBundlePricingArgs>(), 8);
    assert_eq!(seal_bytes.len(), 9);
    assert_eq!(seal_bytes[0], 32);
    assert_eq!(seal_bytes[0], AuctionInstruction::SealBundlePricing as u8);
    assert_eq!(
        SealBundlePricingArgs::try_from(&seal_bytes[1..]).unwrap(),
        seal
    );

    let commit = CommitAuctionSettlementV3Args {
        auction_hash: [6; 32],
        winner_node_pubkey: [7; 32],
    };
    let commit_bytes = commit.to_bytes();
    assert_eq!(size_of::<CommitAuctionSettlementV3Args>(), 64);
    assert_eq!(offset_of!(CommitAuctionSettlementV3Args, auction_hash), 0);
    assert_eq!(
        offset_of!(CommitAuctionSettlementV3Args, winner_node_pubkey),
        32
    );
    assert_eq!(commit_bytes.len(), 65);
    assert_eq!(commit_bytes[0], 33);
    assert_eq!(
        commit_bytes[0],
        AuctionInstruction::CommitAuctionSettlementV3 as u8
    );
    assert_eq!(
        CommitAuctionSettlementV3Args::try_from(&commit_bytes[1..]).unwrap(),
        commit
    );
    assert!(CommitAuctionSettlementV2Args::try_from(&commit_bytes[1..]).is_err());

    // V6 applies to the escrow/page family, not to the legacy Bundle account.
    assert_eq!(bundle_account_len(AccountLayoutVersion::V6), 0);
}

#[test]
fn v6_instruction_account_parsers_preserve_program_account_order() {
    let post_accounts = [1_u8, 2, 3];
    let parsed_post = PostBundlePricingAccounts::try_from(&post_accounts[..]).unwrap();
    assert_eq!(parsed_post.iter_owned().collect::<Vec<_>>(), post_accounts);
    assert!(PostBundlePricingAccounts::try_from(&post_accounts[..2]).is_err());

    let seal_accounts = [1_u8, 2, 3, 4, 5];
    let parsed_seal = SealBundlePricingAccounts::try_from(&seal_accounts[..]).unwrap();
    assert_eq!(parsed_seal.iter_owned().collect::<Vec<_>>(), seal_accounts);
    assert_eq!(parsed_seal.bundle_verifier_pages, &[3, 4, 5]);
    assert!(SealBundlePricingAccounts::try_from(&seal_accounts[..2]).is_err());

    let commit_accounts = [1_u8, 2, 3, 4];
    let parsed_commit = CommitAuctionSettlementV3Accounts::try_from(&commit_accounts[..]).unwrap();
    assert_eq!(
        parsed_commit.iter_owned().collect::<Vec<_>>(),
        commit_accounts
    );

    assert_eq!(MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES, 6);
}
