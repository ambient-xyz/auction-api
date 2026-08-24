use ambient_auction_api::{
    AuctionInstruction, ConfigPolicyV2, ConfigPolicyV2PatchKind, InitConfigPolicyV2Args,
    InstructionBytes, PostBundleResultV2Args, PostBundleResultV3Args, Pubkey,
    SetConfigPolicyV2Args, SlashSmallCreditsArgs, SmallCreditSettings,
};
use bytemuck::Zeroable;
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn small_policy_uses_only_reserved_words_one_and_two() {
    assert_eq!(ConfigPolicyV2::LEN, 1_568);
    assert_eq!(ConfigPolicyV2PatchKind::SMALL_CREDIT_SETTINGS.0, 7);
    assert_eq!(size_of::<InitConfigPolicyV2Args>(), 584);
    assert_eq!(size_of::<SetConfigPolicyV2Args>(), 160);
    assert_eq!(offset_of!(SetConfigPolicyV2Args, small_credit_enabled), 5);
    assert_eq!(offset_of!(SetConfigPolicyV2Args, authority), 32);
    let mut set = SetConfigPolicyV2Args::zeroed();
    set.patch_kind = ConfigPolicyV2PatchKind::SMALL_CREDIT_SETTINGS;
    set.small_credit_enabled = 1;
    set.authority = Pubkey::from([5; 32]);
    let set_bytes = set.to_bytes();
    assert_eq!(set_bytes.len(), 161);
    assert_eq!(set_bytes[0], AuctionInstruction::SetConfigPolicyV2 as u8);
    assert_eq!(set_bytes[6], 1);
    assert_eq!(&set_bytes[33..65], &[5; 32]);
    assert_eq!(InitConfigPolicyV2Args::zeroed().to_bytes().len(), 585);

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

    policy.reserved_words[2][1] = 1;
    assert!(!policy.small_credit_settings_word_is_canonical());
    assert!(!SmallCreditSettings {
        enabled: true,
        mint: Pubkey::default(),
    }
    .validate());
}

#[test]
fn slash_authority_uses_reserved_word_three() {
    assert_eq!(ConfigPolicyV2::LEN, 1_568);
    assert_eq!(ConfigPolicyV2PatchKind::SMALL_CREDIT_SLASH_AUTHORITY.0, 6);
    assert_eq!(size_of::<SetConfigPolicyV2Args>(), 160);
    assert_eq!(size_of::<SlashSmallCreditsArgs>(), 16);

    let mut policy = ConfigPolicyV2::production_default();
    let before = bytemuck::bytes_of(&policy).to_vec();
    assert_eq!(policy.small_credit_slash_authority(), Pubkey::default());

    let authority = Pubkey::from([7; 32]);
    policy.set_small_credit_slash_authority(authority);
    let after = bytemuck::bytes_of(&policy);
    assert_eq!(policy.small_credit_slash_authority(), authority);
    assert_eq!(&after[1_384..1_416], &[7; 32]);
    assert_eq!(&before[..1_384], &after[..1_384]);
    assert_eq!(&before[1_416..], &after[1_416..]);

    let mut patch = SetConfigPolicyV2Args::zeroed();
    patch.patch_kind = ConfigPolicyV2PatchKind::SMALL_CREDIT_SLASH_AUTHORITY;
    patch.authority = authority;
    let encoded_patch = patch.to_bytes();
    assert_eq!(encoded_patch.len(), 161);
    assert_eq!(
        SetConfigPolicyV2Args::try_from(&encoded_patch[1..]),
        Ok(patch)
    );

    let args = SlashSmallCreditsArgs {
        amount: 2,
        expected_token_account_balance: 5,
    };
    let encoded = args.to_bytes();
    assert_eq!(encoded.len(), 17);
    assert_eq!(encoded[0], AuctionInstruction::SlashSmallCredits as u8);
    assert_eq!(&encoded[1..], bytemuck::bytes_of(&args));
}

#[test]
fn v3_post_preserves_the_existing_payload_and_instruction_gaps() {
    for discriminator in [22, 23, 24] {
        assert!(AuctionInstruction::try_from(discriminator).is_err());
    }
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
