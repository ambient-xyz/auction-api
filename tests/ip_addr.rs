//! Wire-compatibility and validation tests for [`IpAddr`].
//!
//! `IpAddr` used to be a `#[repr(C)]` two-variant Rust enum with a hand-written
//! `unsafe impl Pod for IpAddr {}`. `PlaceBidArgs` is read straight out of attacker-controlled
//! instruction data with `bytemuck::try_pod_read_unaligned`, so a bidder could submit any 32-bit tag
//! and materialize an enum value that does not exist; every later `match` on it was undefined
//! behaviour rather than a wrong answer.
//!
//! The replacement is an explicit `u32` tag plus a 16-byte payload — both accept every bit pattern,
//! so the `Pod` impl is now true. These tests pin the two things that makes the change safe to ship:
//! the byte layout is identical to the enum's, and an unrecognised tag fails closed.

use ambient_auction_api::instruction::IpAddr;
use std::mem::{align_of, size_of};
use std::net::{IpAddr as StdIpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn ip_addr_layout_is_unchanged() {
    // Twenty bytes at alignment four, exactly as the former enum occupied, so the offset of every
    // field that follows an `IpAddr` in `PlaceBidArgs` and `Bid` is unchanged.
    assert_eq!(size_of::<IpAddr>(), 20);
    assert_eq!(align_of::<IpAddr>(), 4);
}

#[test]
fn ipv4_round_trips_through_std() {
    let addr = IpAddr::v4([203, 0, 113, 7]);
    assert_eq!(
        addr.to_std(),
        Some(StdIpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)))
    );
    assert_eq!(IpAddr::from(StdIpAddr::V4(Ipv4Addr::new(203, 0, 113, 7))), addr);
}

#[test]
fn ipv6_round_trips_through_std() {
    let segments = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 0x0001];
    let addr = IpAddr::v6(segments);
    let expected = Ipv6Addr::new(
        segments[0],
        segments[1],
        segments[2],
        segments[3],
        segments[4],
        segments[5],
        segments[6],
        segments[7],
    );
    assert_eq!(addr.to_std(), Some(StdIpAddr::V6(expected)));
    assert_eq!(IpAddr::from(StdIpAddr::V6(expected)), addr);
}

/// The byte-level assertions. These are what prove the new struct is a re-description of the old
/// enum's memory rather than a new encoding: an IPv4 address is tag `0` followed by the four octets
/// in order, an IPv6 address is tag `1` followed by the eight `u16` segments little-endian.
#[test]
fn ip_addr_bytes_match_the_previous_enum_encoding() {
    let v4 = IpAddr::v4([10, 1, 2, 3]);
    let bytes = bytemuck::bytes_of(&v4);
    assert_eq!(&bytes[..4], &IpAddr::TAG_V4.to_le_bytes());
    assert_eq!(&bytes[4..8], &[10, 1, 2, 3]);
    // The twelve unused payload bytes are explicitly zeroed. The enum left them as whatever its
    // union padding happened to hold, which leaked uninitialised memory into account data.
    assert_eq!(&bytes[8..], &[0u8; 12]);

    let segments = [1u16, 2, 3, 4, 5, 6, 7, 8];
    let v6 = IpAddr::v6(segments);
    let bytes = bytemuck::bytes_of(&v6);
    assert_eq!(&bytes[..4], &IpAddr::TAG_V6.to_le_bytes());
    let mut expected = Vec::with_capacity(16);
    for segment in segments {
        expected.extend_from_slice(&segment.to_le_bytes());
    }
    assert_eq!(&bytes[4..], expected.as_slice());
}

/// The case the `Pod` enum could not express. A tag no version of this program writes must be
/// reported as "not an address" rather than executed as a variant that does not exist.
#[test]
fn unknown_tag_fails_closed() {
    for tag in [2u32, 7, u32::MAX] {
        let mut bytes = [0u8; size_of::<IpAddr>()];
        bytes[..4].copy_from_slice(&tag.to_le_bytes());
        // Filling the payload keeps the test honest: the rejection must come from the tag, not from
        // the payload happening to be zero.
        bytes[4..].fill(0xAB);

        // Read the way instruction data is actually read: unaligned, straight out of a byte slice.
        let addr: IpAddr = bytemuck::pod_read_unaligned(&bytes);
        assert_eq!(addr.to_std(), None, "tag {tag} must not decode");

        // The infallible conversion degrades to `0.0.0.0`, which the listener's
        // `validate_inference_endpoint` already rejects, so the failure mode is closed rather than
        // exploitable.
        assert_eq!(
            StdIpAddr::from(addr),
            StdIpAddr::V4(Ipv4Addr::UNSPECIFIED)
        );
    }
}

/// Every byte pattern of the payload is a valid address for a valid tag, which is the property the
/// `Pod` impl actually promises. Exercising the extremes documents that there is no remaining
/// unchecked tag range hidden in the payload.
#[test]
fn every_payload_pattern_is_valid_for_a_known_tag() {
    let v4 = IpAddr::v4([255, 255, 255, 255]);
    assert_eq!(
        v4.to_std(),
        Some(StdIpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)))
    );

    let v6 = IpAddr::v6([u16::MAX; 8]);
    assert_eq!(
        v6.to_std(),
        Some(StdIpAddr::V6(Ipv6Addr::from([u16::MAX; 8])))
    );

    let zero = IpAddr::v4([0, 0, 0, 0]);
    assert_eq!(zero.to_std(), Some(StdIpAddr::V4(Ipv4Addr::UNSPECIFIED)));
}
