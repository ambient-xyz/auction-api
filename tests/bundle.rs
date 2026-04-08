use ambient_auction_api::{
    bundle_account_len, parse_bundle_layout, AccountDiscriminator, AccountLayoutVersion,
    BundleDataMut, BundleDataRef, BundleLayoutTrailerV1, BundleStatus, InvalidBundleTransition,
    ParsedAccountLayout, RawBundleData, RawBundleMut, RawBundleRef, RequestBundle, RequestTier,
};
use memoffset::offset_of;
use std::mem::size_of;

#[test]
fn layout_offsets() {
    assert_eq!(offset_of!(RawBundleData, status), 0);
    assert_eq!(offset_of!(RawBundleData, context_length_tier), 8);
    assert_eq!(offset_of!(RawBundleData, expiry_duration_tier), 16);
    assert_eq!(offset_of!(RawBundleData, expiry_slot), 152);
    assert_eq!(offset_of!(RawBundleData, requests_len), 168);
}

#[test]
fn layout_sizes() {
    assert_eq!(size_of::<RequestTier>(), 8);
    assert_eq!(size_of::<BundleStatus>(), 8);

    let request = RawBundleData::default();
    let _: BundleStatus = request.status;
    let _: RequestTier = request.context_length_tier;
    let _: RequestTier = request.expiry_duration_tier;
    let _: u64 = request.expiry_slot;
    let _: u64 = request.requests_len;
}

#[test]
fn request_bundle_alias_matches_legacy_layout() {
    assert_eq!(RequestBundle::LEN, RawBundleData::LEGACY_LEN);
}

#[test]
fn request_bundle_remains_legacy_pod() {
    let raw = RawBundleData {
        expiry_slot: 5,
        requests_len: 2,
        ..Default::default()
    };

    let decoded = bytemuck::try_from_bytes::<RequestBundle>(bytemuck::bytes_of(&raw)).unwrap();
    assert_eq!(decoded, &raw);
}

#[test]
fn current_bundle_bytes_classify_as_legacy_v0() {
    let raw = RawBundleData::default();
    let layout = parse_bundle_layout(bytemuck::bytes_of(&raw)).unwrap();
    assert_eq!(
        layout,
        ParsedAccountLayout::legacy_v0(AccountDiscriminator::Bundle)
    );
    assert!(layout.is_legacy());
}

#[test]
fn oversized_legacy_bundle_bytes_classify_as_legacy_v0() {
    let raw = RawBundleData::default();
    let mut bytes = vec![0xAA; RawBundleData::LEGACY_LEN + 10];
    bytes[..RawBundleData::LEGACY_LEN].copy_from_slice(bytemuck::bytes_of(&raw));

    let layout = parse_bundle_layout(&bytes).unwrap();
    assert_eq!(
        layout,
        ParsedAccountLayout::legacy_v0(AccountDiscriminator::Bundle)
    );
    assert!(layout.is_legacy());
}

#[test]
fn state_view_matches_bundle_status() {
    let mut active = RawBundleData::default();
    assert!(matches!(
        BundleDataRef::from_raw(&active),
        BundleDataRef::Active(_)
    ));

    active.status = BundleStatus::Full;
    assert!(matches!(
        BundleDataRef::from_raw(&active),
        BundleDataRef::Full(_)
    ));

    active.status = BundleStatus::PendingVerification;
    assert!(matches!(
        BundleDataRef::from_raw(&active),
        BundleDataRef::PendingVerification(_)
    ));

    active.status = BundleStatus::Verified;
    assert!(matches!(
        BundleDataRef::from_raw(&active),
        BundleDataRef::Verified(_)
    ));

    active.status = BundleStatus::BadJobOutput;
    assert!(matches!(
        BundleDataRef::from_raw(&active),
        BundleDataRef::BadJobOutput(_)
    ));

    active.status = BundleStatus::Canceled;
    assert!(matches!(
        BundleDataRef::from_raw(&active),
        BundleDataRef::Canceled(_)
    ));
}

#[test]
fn transition_helpers_update_status() {
    let mut raw = RawBundleData::default();
    let raw = BundleDataMut::from_raw(&mut raw)
        .mark_full()
        .unwrap()
        .into_raw();
    assert_eq!(raw.status, BundleStatus::Full);

    let raw = BundleDataMut::from_raw(raw)
        .mark_verified()
        .unwrap()
        .into_raw();
    assert_eq!(raw.status, BundleStatus::Verified);
}

#[test]
fn transition_helpers_reject_invalid_moves() {
    let mut raw = RawBundleData {
        status: BundleStatus::Verified,
        ..Default::default()
    };

    let err = BundleDataMut::from_raw(&mut raw)
        .mark_canceled()
        .unwrap_err();
    assert_eq!(
        err,
        InvalidBundleTransition {
            from: BundleStatus::Verified,
            to: BundleStatus::Canceled,
        }
    );
}

#[test]
fn raw_bundle_views_deref_to_legacy_payload() {
    let raw = RawBundleData {
        requests_len: 3,
        max_context_length: 42,
        ..Default::default()
    };
    let bytes = bytemuck::bytes_of(&raw);

    let parsed = RawBundleRef::from_bytes(bytes).unwrap();
    assert_eq!(parsed.requests_len, 3);
    assert_eq!(parsed.max_context_length, 42);

    let mut mutable_bytes = bytes.to_vec();
    let mut parsed = RawBundleMut::from_bytes(&mut mutable_bytes).unwrap();
    parsed.requests_len = 9;

    let reparsed = RawBundleData::from_bytes(&mutable_bytes).unwrap();
    assert_eq!(reparsed.requests_len, 9);
}

#[test]
fn raw_bundle_views_accept_oversized_legacy_bytes() {
    let raw = RawBundleData {
        requests_len: 3,
        max_context_length: 42,
        ..Default::default()
    };
    let mut bytes = vec![0xAA; RawBundleData::LEGACY_LEN + 10];
    bytes[..RawBundleData::LEGACY_LEN].copy_from_slice(bytemuck::bytes_of(&raw));
    let trailing = bytes[RawBundleData::LEGACY_LEN..].to_vec();

    let parsed = RawBundleRef::from_bytes(&bytes).unwrap();
    assert_eq!(
        parsed.layout(),
        ParsedAccountLayout::legacy_v0(AccountDiscriminator::Bundle)
    );
    assert_eq!(parsed.requests_len, 3);
    assert_eq!(parsed.max_context_length, 42);

    {
        let mut parsed = RawBundleMut::from_bytes(&mut bytes).unwrap();
        parsed.requests_len = 9;
    }

    assert_eq!(&bytes[RawBundleData::LEGACY_LEN..], trailing.as_slice());

    let reparsed = RawBundleData::from_bytes(&bytes).unwrap();
    assert_eq!(reparsed.requests_len, 9);
}

#[test]
fn raw_bundle_mut_mark_helpers_match_state_transitions() {
    let raw = RawBundleData::default();
    let mut bytes = bytemuck::bytes_of(&raw).to_vec();

    {
        let mut parsed = RawBundleMut::from_bytes(&mut bytes).unwrap();
        parsed.mark_full().unwrap();
        parsed.mark_verified().unwrap();
    }

    let reparsed = RawBundleData::from_bytes(&bytes).unwrap();
    assert_eq!(reparsed.status, BundleStatus::Verified);
}

#[test]
fn write_legacy_bytes_preserves_v1_trailer() {
    let raw = RawBundleData {
        expiry_slot: 5,
        requests_len: 2,
        ..Default::default()
    };
    let trailer = BundleLayoutTrailerV1::new();
    let mut bytes = vec![0_u8; bundle_account_len(AccountLayoutVersion::V1)];
    bytes[RawBundleData::LEGACY_LEN..].copy_from_slice(bytemuck::bytes_of(&trailer));

    assert!(raw.write_legacy_bytes(&mut bytes));
    assert_eq!(
        &bytes[..RawBundleData::LEGACY_LEN],
        bytemuck::bytes_of(&raw)
    );
    assert_eq!(
        &bytes[RawBundleData::LEGACY_LEN..],
        bytemuck::bytes_of(&trailer)
    );
    assert_eq!(
        parse_bundle_layout(&bytes),
        Some(ParsedAccountLayout::new(
            AccountDiscriminator::Bundle,
            AccountLayoutVersion::V1
        ))
    );
}

#[test]
fn legacy_helpers_support_v1_bundle_bytes() {
    let raw = RawBundleData {
        expiry_slot: 5,
        requests_len: 0,
        ..Default::default()
    };
    let mut bytes = vec![0_u8; bundle_account_len(AccountLayoutVersion::V1)];
    bytes[..RawBundleData::LEGACY_LEN].copy_from_slice(bytemuck::bytes_of(&raw));
    bytes[RawBundleData::LEGACY_LEN..]
        .copy_from_slice(bytemuck::bytes_of(&BundleLayoutTrailerV1::new()));

    assert_eq!(RawBundleData::is_expired_from_bytes(&bytes, 5), Some(true));
    assert!(RawBundleData::cancel_bundle_from_bytes(&mut bytes));

    let canceled =
        bytemuck::try_from_bytes::<RawBundleData>(&bytes[..RawBundleData::LEGACY_LEN]).unwrap();
    assert_eq!(canceled.status, BundleStatus::Canceled);
}
