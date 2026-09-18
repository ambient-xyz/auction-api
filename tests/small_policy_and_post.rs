use ambient_auction_api::{
    AuctionInstruction, ConfigPolicyV2, ConfigPolicyV2PatchKind, InitConfigPolicySmallV3Args,
    InstructionBytes, PostBundleResultV2Args, PostBundleResultV3Args, Pubkey,
    SetConfigPolicySmallV3Args, SlashSmallCreditsArgs, SmallCreditSettings,
};
use bytemuck::Zeroable;
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn small_policy_preserves_original_byte_offsets() {
    assert_eq!(ConfigPolicyV2::LEN, 1_568);
    assert_eq!(ConfigPolicyV2PatchKind::SMALL_CREDIT_SETTINGS.0, 7);
    assert_eq!(size_of::<InitConfigPolicySmallV3Args>(), 584);
    assert_eq!(size_of::<SetConfigPolicySmallV3Args>(), 160);
    assert_eq!(
        offset_of!(SetConfigPolicySmallV3Args, small_credit_enabled),
        5
    );
    assert_eq!(offset_of!(SetConfigPolicySmallV3Args, authority), 32);
    let mut set = SetConfigPolicySmallV3Args::zeroed();
    set.patch_kind = ConfigPolicyV2PatchKind::SMALL_CREDIT_SETTINGS;
    set.small_credit_enabled = 1;
    set.authority = Pubkey::from([5; 32]);
    let set_bytes = set.to_bytes();
    assert_eq!(set_bytes.len(), 161);
    assert_eq!(set_bytes[0], AuctionInstruction::SetConfigPolicyV2 as u8);
    assert_eq!(set_bytes[6], 1);
    assert_eq!(&set_bytes[33..65], &[5; 32]);
    assert_eq!(InitConfigPolicySmallV3Args::zeroed().to_bytes().len(), 585);

    let mut policy = ConfigPolicyV2::production_default();
    let settings = SmallCreditSettings {
        enabled: true,
        mint: Pubkey::from([5; 32]),
    };
    assert!(settings.validate());
    policy.set_small_credit_settings(settings);
    assert_eq!(policy.small_credit_settings(), settings);
    assert!(policy.small_credit_settings_word_is_canonical());

    let bytes = bytemuck::bytes_of(&policy);
    assert_eq!(&bytes[1_320..1_352], &[5; 32]);
    assert_eq!(bytes[1_352], 1);
    assert!(bytes[1_353..1_384].iter().all(|byte| *byte == 0));

    policy.reserved_words[1][1] = 1;
    assert!(!policy.small_credit_settings_word_is_canonical());
    assert!(!SmallCreditSettings {
        enabled: true,
        mint: Pubkey::default(),
    }
    .validate());
}

#[test]
fn slash_settings_preserve_original_byte_offsets() {
    assert_eq!(ConfigPolicyV2::LEN, 1_568);
    assert_eq!(ConfigPolicyV2PatchKind::SMALL_CREDIT_SLASH_AUTHORITY.0, 6);
    assert_eq!(size_of::<SetConfigPolicySmallV3Args>(), 160);
    assert_eq!(size_of::<SlashSmallCreditsArgs>(), 16);

    let mut policy = ConfigPolicyV2::production_default();
    let before = bytemuck::bytes_of(&policy).to_vec();
    assert_eq!(policy.small_credit_slash_authority(), Pubkey::default());
    assert_eq!(policy.small_credit_slash_sequence(), 0);
    assert!(policy.small_credit_slash_sequence_word_is_canonical());

    let authority = Pubkey::from([7; 32]);
    policy.set_small_credit_slash_authority(authority);
    let after = bytemuck::bytes_of(&policy);
    assert_eq!(policy.small_credit_slash_authority(), authority);
    assert_eq!(&after[1_384..1_416], &[7; 32]);
    assert_eq!(&before[..1_384], &after[..1_384]);
    assert_eq!(&before[1_416..], &after[1_416..]);

    policy.set_small_credit_slash_sequence(5);
    let after = bytemuck::bytes_of(&policy);
    assert_eq!(policy.small_credit_slash_sequence(), 5);
    assert_eq!(&after[1_416..1_424], &5_u64.to_le_bytes());
    assert!(after[1_424..1_448].iter().all(|byte| *byte == 0));
    policy.reserved_words[3][8] = 1;
    assert!(!policy.small_credit_slash_sequence_word_is_canonical());

    let mut patch = SetConfigPolicySmallV3Args::zeroed();
    patch.patch_kind = ConfigPolicyV2PatchKind::SMALL_CREDIT_SLASH_AUTHORITY;
    patch.authority = authority;
    let encoded_patch = patch.to_bytes();
    assert_eq!(encoded_patch.len(), 161);
    assert_eq!(
        SetConfigPolicySmallV3Args::try_from(&encoded_patch[1..]),
        Ok(patch)
    );

    let args = SlashSmallCreditsArgs {
        amount: 2,
        sequence: 5,
    };
    let encoded = args.to_bytes();
    assert_eq!(encoded.len(), 17);
    assert_eq!(encoded[0], AuctionInstruction::SlashSmallCredits as u8);
    assert_eq!(&encoded[1..], bytemuck::bytes_of(&args));
}

#[test]
fn v3_post_preserves_the_existing_payload_and_instruction_gaps() {
    assert_eq!(AuctionInstruction::DisputeBundleVerificationV2 as u8, 22);
    assert_eq!(AuctionInstruction::SelectBundleVerifiersV2 as u8, 23);
    assert!(AuctionInstruction::try_from(24_u8).is_err());
    assert_eq!(size_of::<PostBundleResultV2Args>(), 816);
    assert_eq!(size_of::<PostBundleResultV3Args>(), 864);
    assert_eq!(offset_of!(PostBundleResultV3Args, input_tokens), 816);

    let mut args = PostBundleResultV3Args::zeroed();
    args.post.result_hash = [9; 32];
    args.input_tokens = [1, 2, 3, 4, 5, 6];
    let encoded = args.to_bytes();
    assert_eq!(encoded[0], AuctionInstruction::PostBundleResultV2 as u8);
    assert_eq!(&encoded[1..817], bytemuck::bytes_of(&args.post));
    assert_eq!(&encoded[817..], bytemuck::bytes_of(&args.input_tokens));
    assert!(PostBundleResultV2Args::try_from(&encoded[1..]).is_err());
    assert!(PostBundleResultV3Args::try_from(&encoded[1..817]).is_err());

    let post = args.post.to_bytes();
    assert_eq!(post[0], AuctionInstruction::PostBundleResultV2 as u8);
    assert_eq!(&post[1..], bytemuck::bytes_of(&args.post));
}

#[test]
fn small_configuration_writes_preserve_v2_dispute_terms() {
    let mut policy = ConfigPolicyV2 {
        missed_verification_dispute_window_slots: 11,
        dispute_verification_window_slots: 22,
        paid_verification_dispute_window_slots: 33,
        paid_verification_dispute_bond_lamports: 44,
        ..Default::default()
    };
    let before = bytemuck::bytes_of(&policy).to_vec();
    policy.set_small_credit_settings(SmallCreditSettings {
        enabled: true,
        mint: Pubkey::from([5; 32]),
    });
    policy.set_small_credit_slash_authority(Pubkey::from([7; 32]));
    policy.set_small_credit_slash_sequence(9);
    assert_eq!(&bytemuck::bytes_of(&policy)[..1320], &before[..1320]);
    assert_eq!(&bytemuck::bytes_of(&policy)[1448..], &before[1448..]);
}

#[test]
fn configuration_formats_remain_distinct() {
    use ambient_auction_api::{InitConfigPolicyV2Args, SetConfigPolicyV2Args};
    assert_eq!(size_of::<InitConfigPolicyV2Args>(), 616);
    assert_eq!(size_of::<SetConfigPolicyV2Args>(), 192);
    assert_eq!(offset_of!(SetConfigPolicyV2Args, authority), 64);
    assert!(SetConfigPolicyV2Args::try_from(bytemuck::bytes_of(
        &SetConfigPolicySmallV3Args::zeroed()
    ))
    .is_err());
    assert!(SetConfigPolicySmallV3Args::try_from(bytemuck::bytes_of(
        &SetConfigPolicyV2Args::zeroed()
    ))
    .is_err());
    assert!(InitConfigPolicyV2Args::try_from(bytemuck::bytes_of(
        &InitConfigPolicySmallV3Args::zeroed()
    ))
    .is_err());
    assert!(InitConfigPolicySmallV3Args::try_from(bytemuck::bytes_of(
        &InitConfigPolicyV2Args::zeroed()
    ))
    .is_err());
}
