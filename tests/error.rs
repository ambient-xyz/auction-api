use ambient_auction_api::error::AuctionError;

#[test]
fn existing_error_codes_still_decode_to_the_same_variants() {
    assert_eq!(AuctionError::try_from_code(0), Ok(AuctionError::Unknown));
    assert_eq!(
        AuctionError::try_from_code(41),
        Ok(AuctionError::InvalidVoteAccount)
    );
    assert_eq!(
        AuctionError::try_from_code(59),
        Ok(AuctionError::UnauthorizedAccountLayoutVersion)
    );
}

#[test]
fn new_error_codes_decode_strictly() {
    assert_eq!(
        AuctionError::try_from_code(60),
        Ok(AuctionError::InvalidVerifierCount)
    );
    assert_eq!(
        AuctionError::try_from_code(61),
        Ok(AuctionError::InvalidTierConfig)
    );
    assert_eq!(
        AuctionError::try_from_code(62),
        Ok(AuctionError::SettlementDeadlinePassed)
    );
}

#[test]
fn unknown_error_codes_are_rejected_instead_of_defaulting_to_unknown() {
    assert_eq!(AuctionError::try_from_code(15), Err(15));
    assert_eq!(AuctionError::try_from_code(999), Err(999));
    assert_eq!(AuctionError::describe_code(15), None);
}

#[test]
fn error_names_and_messages_are_stable_for_representative_variants() {
    assert_eq!(
        AuctionError::InvalidVoteAccount.name(),
        "InvalidVoteAccount"
    );
    assert_eq!(
        AuctionError::InvalidVoteAccount.message(),
        "Vote account does not match the expected node identity"
    );
    assert_eq!(AuctionError::InvalidTierConfig.name(), "InvalidTierConfig");
    assert_eq!(
        AuctionError::InvalidTierConfig.message(),
        "Tier config is invalid"
    );
}

#[test]
fn describe_code_returns_messages_for_known_variants() {
    assert_eq!(
        AuctionError::describe_code(AuctionError::InvalidVerifierQuorum.code()),
        Some("Verifier quorum is invalid")
    );
    assert_eq!(
        AuctionError::describe_code(AuctionError::SettlementDeadlinePassed.code()),
        Some("Settlement deadline has passed")
    );
}
