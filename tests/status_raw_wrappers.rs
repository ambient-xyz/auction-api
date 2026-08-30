//! Wire-compatibility and validation tests for the `*Raw` status/tier wrappers.
//!
//! Every one of these types replaced a `#[repr(u64)]` enum that carried a hand-written
//! `unsafe impl Pod`. That impl claimed every one of the 2^64 bit patterns was a valid variant when
//! only a handful were, so account data or instruction data holding any other discriminant made the
//! following `match` undefined behaviour. The wrappers are `#[repr(transparent)]` over `u64`, so
//! they are genuinely `Pod`, and validation moved into `TryFrom`.
//!
//! Two properties are asserted here, and they are the two the refactor has to hold:
//!
//! 1. **Layout preservation.** Each wrapper is exactly eight bytes and the little-endian bytes of a
//!    valid discriminant are unchanged, so `Auction`, `Bid`, `JobRequest`, `RawBundleData`,
//!    `BundleRegistry`, `InitBundleArgs` and `CancelBundleArgs` keep byte-identical wire formats.
//!    Existing on-chain accounts, PDA seeds and `memcmp` filter offsets are unaffected.
//! 2. **Validation.** A discriminant no released program version writes is reported as a typed
//!    error rather than materialized as a variant that does not exist.

use ambient_auction_api::{
    error::AuctionError, AuctionStatus, AuctionStatusRaw, BidStatus, BidStatusRaw, BundleStatus,
    BundleStatusRaw, JobRequestStatus, JobRequestStatusRaw, RequestTier, RequestTierRaw,
};
use bytemuck::cast;
use std::mem::{align_of, size_of};

/// Round-trips every variant this program writes and asserts the stored bytes are the enum's own
/// little-endian discriminant. This is the assertion that makes the refactor a type change rather
/// than a wire-format change.
macro_rules! assert_round_trip {
    ($enum:ty, $raw:ty, [$($variant:expr),+ $(,)?]) => {{
        assert_eq!(size_of::<$raw>(), size_of::<u64>());
        assert_eq!(align_of::<$raw>(), align_of::<u64>());
        $(
            let raw = <$raw>::from($variant);
            assert_eq!(<$enum>::try_from(raw), Ok($variant));
            // `as_u64` is the stored discriminant, and `u64::from` is the enum's own. Equal values
            // plus equal size means equal bytes on every target.
            assert_eq!(raw.as_u64(), u64::from($variant));
            assert_eq!(cast::<$raw, u64>(raw).to_le_bytes(), u64::from($variant).to_le_bytes());
        )+
    }};
}

#[test]
fn auction_status_round_trips_and_preserves_layout() {
    assert_round_trip!(
        AuctionStatus,
        AuctionStatusRaw,
        [
            AuctionStatus::Active,
            AuctionStatus::RevealingBids,
            AuctionStatus::Ended,
            AuctionStatus::Canceled,
        ]
    );
}

#[test]
fn bid_status_round_trips_and_preserves_layout() {
    assert_round_trip!(
        BidStatus,
        BidStatusRaw,
        [BidStatus::Concealed, BidStatus::Revealed]
    );
}

#[test]
fn bundle_status_round_trips_and_preserves_layout() {
    assert_round_trip!(
        BundleStatus,
        BundleStatusRaw,
        [
            BundleStatus::Active,
            BundleStatus::Full,
            BundleStatus::PendingVerification,
            BundleStatus::Verified,
            BundleStatus::BadJobOutput,
            BundleStatus::Canceled,
        ]
    );
}

#[test]
fn job_request_status_round_trips_and_preserves_layout() {
    assert_round_trip!(
        JobRequestStatus,
        JobRequestStatusRaw,
        [
            JobRequestStatus::WaitingForOutput,
            JobRequestStatus::OutputReceived,
            JobRequestStatus::OutputVerified,
        ]
    );
}

#[test]
fn request_tier_round_trips_and_preserves_layout() {
    // `RequestTier`'s discriminants are deliberately not in declaration order
    // (`Eco = 0, Small = 3, Standard = 1, Pro = 2, Large = 4`), which is exactly why the byte-level
    // assertion above is worth making: a wrapper that renumbered them would silently repoint every
    // stored tier and every tier-seeded PDA.
    assert_round_trip!(
        RequestTier,
        RequestTierRaw,
        [
            RequestTier::Eco,
            RequestTier::Small,
            RequestTier::Standard,
            RequestTier::Pro,
            RequestTier::Large,
        ]
    );
    assert_eq!(RequestTierRaw::new(RequestTier::Small).as_u64(), 3);
    assert_eq!(RequestTierRaw::new(RequestTier::Standard).as_u64(), 1);
}

/// The default of each wrapper must be the zero bit pattern, because `Zeroable`/`Default` is what a
/// freshly allocated account holds before the program writes to it.
#[test]
fn raw_wrapper_defaults_are_the_zero_discriminant() {
    assert_eq!(AuctionStatusRaw::default().as_u64(), 0);
    assert_eq!(BidStatusRaw::default().as_u64(), 0);
    assert_eq!(BundleStatusRaw::default().as_u64(), 0);
    assert_eq!(JobRequestStatusRaw::default().as_u64(), 0);
    assert_eq!(RequestTierRaw::default().as_u64(), 0);

    // And the zero pattern must decode to the state a zeroed account is supposed to mean.
    assert_eq!(
        AuctionStatus::try_from(AuctionStatusRaw::default()),
        Ok(AuctionStatus::Active)
    );
    assert_eq!(
        BidStatus::try_from(BidStatusRaw::default()),
        Ok(BidStatus::Concealed)
    );
    assert_eq!(
        BundleStatus::try_from(BundleStatusRaw::default()),
        Ok(BundleStatus::Active)
    );
    assert_eq!(
        JobRequestStatus::try_from(JobRequestStatusRaw::default()),
        Ok(JobRequestStatus::WaitingForOutput)
    );
    assert_eq!(
        RequestTier::try_from(RequestTierRaw::default()),
        Ok(RequestTier::Eco)
    );
}

/// The case a `Pod` enum could not express. Each of these discriminants is one the program never
/// writes; a caller or a corrupt account can still present them, and each must produce a typed error
/// rather than a variant that does not exist.
#[test]
fn raw_wrappers_reject_unknown_discriminants() {
    assert_eq!(
        AuctionStatus::try_from(cast::<u64, AuctionStatusRaw>(4)),
        Err(AuctionError::InvalidAuctionStatus)
    );
    assert_eq!(
        AuctionStatus::try_from(cast::<u64, AuctionStatusRaw>(u64::MAX)),
        Err(AuctionError::InvalidAuctionStatus)
    );

    assert_eq!(
        BidStatus::try_from(cast::<u64, BidStatusRaw>(2)),
        Err(AuctionError::InvalidBidStatus)
    );

    // `1` is a gap in `BundleStatus` (`Active = 0`, `Full = 2`), so it is the sharpest single value
    // to test: it is small, plausible, and still not a state.
    assert_eq!(
        BundleStatus::try_from(cast::<u64, BundleStatusRaw>(1)),
        Err(AuctionError::InvalidBundleStatus)
    );
    assert_eq!(
        BundleStatus::try_from(cast::<u64, BundleStatusRaw>(7)),
        Err(AuctionError::InvalidBundleStatus)
    );

    assert_eq!(
        JobRequestStatus::try_from(cast::<u64, JobRequestStatusRaw>(3)),
        Err(AuctionError::InvalidJobRequestStatus)
    );

    assert_eq!(
        RequestTier::try_from(cast::<u64, RequestTierRaw>(5)),
        Err(AuctionError::InvalidRequestTier)
    );
    assert_eq!(
        RequestTier::try_from(cast::<u64, RequestTierRaw>(u64::MAX)),
        Err(AuctionError::InvalidRequestTier)
    );
}

/// `as_u64` has to keep reporting the stored discriminant even when it is invalid, because that is
/// the only way a decoder or an operator can tell *what* a corrupt account holds.
#[test]
fn as_u64_reports_invalid_discriminants_for_diagnostics() {
    assert_eq!(cast::<u64, BundleStatusRaw>(1).as_u64(), 1);
    assert_eq!(cast::<u64, RequestTierRaw>(9_999).as_u64(), 9_999);
    assert_eq!(
        cast::<u64, JobRequestStatusRaw>(u64::MAX).as_u64(),
        u64::MAX
    );
}

/// The wrappers serialize as the enum's own name, so JSON emitted by the decoder and by any consumer
/// of the `serde` feature is byte-identical to what the `Pod` enums produced.
#[cfg(feature = "serde")]
#[test]
fn raw_wrapper_serde_matches_enum_representation() {
    assert_eq!(
        serde_json::to_string(&BundleStatusRaw::new(BundleStatus::PendingVerification)).unwrap(),
        "\"PendingVerification\""
    );
    assert_eq!(
        serde_json::to_string(&RequestTierRaw::new(RequestTier::Large)).unwrap(),
        "\"Large\""
    );

    let deserialized: RequestTierRaw = serde_json::from_str("\"Small\"").unwrap();
    assert_eq!(RequestTier::try_from(deserialized), Ok(RequestTier::Small));

    // An invalid stored discriminant becomes a serialization error rather than a fabricated status.
    assert!(serde_json::to_string(&cast::<u64, BundleStatusRaw>(1)).is_err());
}
