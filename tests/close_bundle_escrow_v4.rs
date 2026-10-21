use ambient_auction_api::{
    instruction::AuctionInstruction, CloseBundleEscrowV4Accounts, CloseBundleEscrowV4Args,
    InstructionAccounts, InstructionBytes,
};

#[test]
fn close_bundle_escrow_v4_wire_format_is_stable() {
    assert_eq!(AuctionInstruction::CloseBundleEscrowV4 as u8, 39);
    assert_eq!(
        AuctionInstruction::try_from(39u8),
        Ok(AuctionInstruction::CloseBundleEscrowV4)
    );
    assert_eq!(std::mem::size_of::<CloseBundleEscrowV4Args>(), 0);
    assert_eq!(CloseBundleEscrowV4Args {}.to_bytes(), vec![39]);
    assert_eq!(
        CloseBundleEscrowV4Args::try_from(&[][..]).unwrap(),
        CloseBundleEscrowV4Args {}
    );
    assert!(CloseBundleEscrowV4Args::try_from(&[0u8][..]).is_err());
}

#[test]
fn close_bundle_escrow_v4_accounts_are_payer_then_escrow() {
    let keys = [1u8, 2, 3];
    let accounts = CloseBundleEscrowV4Accounts::try_from(&keys[..]).unwrap();
    assert_eq!(*accounts.payer, 1);
    assert_eq!(*accounts.bundle_escrow, 2);
    assert_eq!(accounts.iter().copied().collect::<Vec<_>>(), vec![1, 2]);
    assert!(CloseBundleEscrowV4Accounts::try_from(&keys[..1]).is_err());
}
