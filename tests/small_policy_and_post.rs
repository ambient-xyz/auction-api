use ambient_auction_api::{
    AuctionInstruction, ConfigPolicyV2, ConfigPolicyV2PatchKind, InitConfigPolicyV2Args,
    InstructionBytes, PostBundleResultV2Args, PostSmallBundleResultV2Args, Pubkey,
    SetConfigPolicyV2Args, SmallCreditSettings,
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
    assert_eq!(&bytes[1_352..1_354], &[0; 2]);
    assert_eq!(bytes[1_354], 1);
    assert!(bytes[1_355..1_384].iter().all(|byte| *byte == 0));

    policy.reserved_words[2][0] = 1;
    assert!(!policy.small_credit_settings_word_is_canonical());
    assert!(!SmallCreditSettings {
        enabled: true,
        mint: Pubkey::default(),
    }
    .validate());
}

#[test]
fn small_post_preserves_the_existing_payload_and_instruction_gaps() {
    assert!(AuctionInstruction::try_from(22).is_err());
    assert!(AuctionInstruction::try_from(23).is_err());
    assert_eq!(AuctionInstruction::PostSmallBundleResultV2 as u8, 24);
    assert_eq!(size_of::<PostBundleResultV2Args>(), 816);
    assert_eq!(size_of::<PostSmallBundleResultV2Args>(), 864);
    assert_eq!(offset_of!(PostSmallBundleResultV2Args, input_tokens), 816);

    let mut args = PostSmallBundleResultV2Args::zeroed();
    args.post.result_hash = [9; 32];
    args.input_tokens = [1, 2, 3, 4, 5, 6];
    let encoded = args.to_bytes();
    assert_eq!(encoded[0], 24);
    assert_eq!(&encoded[1..817], bytemuck::bytes_of(&args.post));
    assert_eq!(&encoded[817..], bytemuck::bytes_of(&args.input_tokens));

    let post = args.post.to_bytes();
    assert_eq!(post[0], AuctionInstruction::PostBundleResultV2 as u8);
    assert_eq!(&post[1..], bytemuck::bytes_of(&args.post));
}
