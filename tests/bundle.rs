use ambient_auction_api::{
    bundle_account_len, parse_bundle_layout, AccountDiscriminator, AccountLayoutVersion,
    BundleDataMut, BundleDataRef, BundleLayoutTrailerV1, BundleStatus, BundleStatusRaw,
    InvalidBundleTransition, ParsedAccountLayout, RawBundleData, RawBundleMut, RawBundleRef,
    RequestBundle, RequestTier, RequestTierRaw,
};
use memoffset::offset_of;
use std::mem::size_of;

#[cfg(test)]
mod account_keys_tests;

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
    // The validated raw wrappers are `#[repr(transparent)]` over `u64`, so they occupy exactly the
    // eight bytes the enums used to. Asserting on both the wrapper and the enum keeps the layout
    // guarantee explicit: the wrapper is what the account stores, the enum is what callers read.
    assert_eq!(size_of::<RequestTier>(), 8);
    assert_eq!(size_of::<RequestTierRaw>(), 8);
    assert_eq!(size_of::<BundleStatus>(), 8);
    assert_eq!(size_of::<BundleStatusRaw>(), 8);

    let request = RawBundleData::default();
    let _: BundleStatusRaw = request.status;
    let _: RequestTierRaw = request.context_length_tier;
    let _: RequestTierRaw = request.expiry_duration_tier;
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
    // `from_raw` is now the single validation gate: it converts the stored discriminant once and
    // every later `status()` read is derived from the variant. Each status here is one this program
    // writes, so `expect` documents that the gate cannot fail for these inputs.
    let mut active = RawBundleData::default();
    assert!(matches!(
        BundleDataRef::from_raw(&active).expect("default status is Active"),
        BundleDataRef::Active(_)
    ));

    active.status = BundleStatus::Full.into();
    assert!(matches!(
        BundleDataRef::from_raw(&active).expect("Full is a valid status"),
        BundleDataRef::Full(_)
    ));

    active.status = BundleStatus::PendingVerification.into();
    assert!(matches!(
        BundleDataRef::from_raw(&active).expect("PendingVerification is a valid status"),
        BundleDataRef::PendingVerification(_)
    ));

    active.status = BundleStatus::Verified.into();
    assert!(matches!(
        BundleDataRef::from_raw(&active).expect("Verified is a valid status"),
        BundleDataRef::Verified(_)
    ));

    active.status = BundleStatus::BadJobOutput.into();
    assert!(matches!(
        BundleDataRef::from_raw(&active).expect("BadJobOutput is a valid status"),
        BundleDataRef::BadJobOutput(_)
    ));

    active.status = BundleStatus::Canceled.into();
    assert!(matches!(
        BundleDataRef::from_raw(&active).expect("Canceled is a valid status"),
        BundleDataRef::Canceled(_)
    ));
}

/// A discriminant no released program version writes must be rejected at the gate rather than
/// matched as a state that does not exist. This is the case a `Pod` enum could not express: it
/// would have produced an invalid `BundleStatus` value and made every later `match` undefined
/// behaviour.
#[test]
fn state_view_rejects_unknown_status_discriminant() {
    let mut raw = RawBundleData::default();
    // `1` is deliberately not a `BundleStatus`; the enum skips it (`Active = 0`, `Full = 2`).
    raw.status = bytemuck::cast::<u64, BundleStatusRaw>(1);
    // Matched rather than compared with `assert_eq!` so the test does not require `PartialEq`/`Debug`
    // on the view types themselves.
    assert!(matches!(
        BundleDataRef::from_raw(&raw),
        Err(ambient_auction_api::error::AuctionError::InvalidBundleStatus)
    ));

    raw.status = bytemuck::cast::<u64, BundleStatusRaw>(u64::MAX);
    assert!(matches!(
        BundleDataMut::from_raw(&mut raw),
        Err(ambient_auction_api::error::AuctionError::InvalidBundleStatus)
    ));
}

#[test]
fn transition_helpers_update_status() {
    let mut raw = RawBundleData::default();
    let raw = BundleDataMut::from_raw(&mut raw)
        .expect("default status is Active")
        .mark_full()
        .unwrap()
        .into_raw();
    assert_eq!(raw.status, BundleStatusRaw::new(BundleStatus::Full));

    let raw = BundleDataMut::from_raw(raw)
        .expect("Full is a valid status")
        .mark_verified()
        .unwrap()
        .into_raw();
    assert_eq!(raw.status, BundleStatusRaw::new(BundleStatus::Verified));
}

#[test]
fn transition_helpers_reject_invalid_moves() {
    let mut raw = RawBundleData {
        status: BundleStatus::Verified.into(),
        ..Default::default()
    };

    let err = BundleDataMut::from_raw(&mut raw)
        .expect("Verified is a valid status")
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
    assert_eq!(reparsed.status, BundleStatusRaw::new(BundleStatus::Verified));
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
    assert_eq!(canceled.status, BundleStatusRaw::new(BundleStatus::Canceled));
}
