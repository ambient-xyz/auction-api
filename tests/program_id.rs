#[cfg(not(feature = "program-id-override"))]
#[test]
fn canonical_program_id_is_the_default() {
    assert_eq!(
        ambient_auction_api::ID,
        five8_const::decode_32_const("Auction111111111111111111111111111111111111")
    );
}

#[cfg(feature = "program-id-override")]
#[test]
fn program_id_matches_the_build_environment() {
    assert_eq!(
        ambient_auction_api::ID,
        five8_const::decode_32_const(env!("AMBIENT_AUCTION_PROGRAM_ID"))
    );
}
