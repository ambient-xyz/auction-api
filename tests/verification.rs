use ambient_auction_api::{error::AuctionError, JobVerificationState, JobVerificationStateRaw};
use bytemuck::cast;

#[test]
fn verification_state_round_trips_through_raw_wrapper() {
    for state in [
        JobVerificationState::NotStarted,
        JobVerificationState::InProgress,
        JobVerificationState::Completed,
    ] {
        let raw = JobVerificationStateRaw::from(state);
        assert_eq!(JobVerificationState::try_from(raw), Ok(state));
    }
}

#[test]
fn verification_state_rejects_invalid_raw_values() {
    let raw = cast::<u64, JobVerificationStateRaw>(99);
    assert_eq!(
        JobVerificationState::try_from(raw),
        Err(AuctionError::InvalidJobVerificationState)
    );
}

#[test]
fn verification_state_raw_matches_u64_layout() {
    assert_eq!(
        std::mem::size_of::<JobVerificationStateRaw>(),
        std::mem::size_of::<u64>()
    );
}

#[cfg(feature = "serde")]
#[test]
fn verification_state_raw_serde_matches_enum_representation() {
    let raw = JobVerificationStateRaw::from(JobVerificationState::Completed);
    let serialized = serde_json::to_string(&raw).unwrap();
    assert_eq!(serialized, "\"Completed\"");

    let deserialized: JobVerificationStateRaw = serde_json::from_str("\"InProgress\"").unwrap();
    assert_eq!(
        JobVerificationState::try_from(deserialized),
        Ok(JobVerificationState::InProgress)
    );
}
