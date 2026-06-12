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
    let new_errors = [
        (60, AuctionError::InvalidVerifierCount),
        (61, AuctionError::InvalidTierConfig),
        (62, AuctionError::SettlementDeadlinePassed),
        (63, AuctionError::AuctionCreditsExceedMax),
        (15, AuctionError::InvalidOpenBundleEscrowV2Args),
        (64, AuctionError::InvalidConfigPolicyV2Args),
        (65, AuctionError::InvalidSettlementV2Args),
        (66, AuctionError::ResultDeadlinePassed),
        (67, AuctionError::VerificationDeadlinePassed),
        (68, AuctionError::ClaimDeadlinePassed),
        (69, AuctionError::InvalidWinnerNode),
        (70, AuctionError::InvalidRefundRecipient),
        (71, AuctionError::InvalidVerifierPageV2Input),
        (72, AuctionError::InvalidVerifierRewardV2),
        (73, AuctionError::InvalidVerificationVerdict),
        (74, AuctionError::InvalidPostedResultV2),
        (75, AuctionError::InvalidVerifierPagesSummary),
    ];

    for (code, error) in new_errors {
        assert_eq!(AuctionError::try_from_code(code), Ok(error));
        assert_eq!(error.code(), code);
        assert_eq!(AuctionError::describe_code(code), Some(error.message()));
        assert!(!error.name().is_empty());
        assert!(!error.message().is_empty());
    }
}

#[test]
fn unknown_error_codes_are_rejected_instead_of_defaulting_to_unknown() {
    assert_eq!(AuctionError::try_from_code(999), Err(999));
    assert_eq!(AuctionError::describe_code(999), None);
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
    assert_eq!(
        AuctionError::AuctionCreditsExceedMax.name(),
        "AuctionCreditsExceedMax"
    );
    assert_eq!(
        AuctionError::AuctionCreditsExceedMax.message(),
        "Auction credits exceed the configured maximum"
    );
    assert_eq!(
        AuctionError::InvalidOpenBundleEscrowV2Args.name(),
        "InvalidOpenBundleEscrowV2Args"
    );
    assert_eq!(
        AuctionError::InvalidOpenBundleEscrowV2Args.message(),
        "Open bundle escrow v2 arguments are invalid"
    );
    assert_eq!(
        AuctionError::InvalidVerifierPagesSummary.name(),
        "InvalidVerifierPagesSummary"
    );
    assert_eq!(
        AuctionError::InvalidVerifierPagesSummary.message(),
        "Page-backed verification summary is invalid"
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
    assert_eq!(
        AuctionError::describe_code(AuctionError::AuctionCreditsExceedMax.code()),
        Some("Auction credits exceed the configured maximum")
    );
}
