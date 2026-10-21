use ambient_auction_api::{
    CommitSmallAuctionSettlementV4Accounts, CommitSmallAuctionSettlementV4Args,
    InstructionAccounts, InstructionBytes, OpenSmallBundleEscrowV4Args,
};
use bytemuck::Zeroable;

#[test]
fn small_instruction_bytes_and_accounts_are_stable() {
    let open = OpenSmallBundleEscrowV4Args {
        bundle_version: 7,
        winner_node_pubkey: [9; 32],
        ..OpenSmallBundleEscrowV4Args::zeroed()
    };
    let bytes = open.to_bytes();
    assert_eq!(bytes.len(), 161);
    assert_eq!(bytes[0], 34);
    assert_eq!(&bytes[1..5], &7u32.to_le_bytes());
    assert_eq!(&bytes[129..161], &[9; 32]);
    assert_eq!(
        OpenSmallBundleEscrowV4Args::try_from(&bytes[1..]).unwrap(),
        open
    );

    let commit = CommitSmallAuctionSettlementV4Args {
        auction_hash: [3; 32],
    };
    let bytes = commit.to_bytes();
    assert_eq!(bytes, [vec![35], vec![3; 32]].concat());
    assert_eq!(
        CommitSmallAuctionSettlementV4Args::try_from(&bytes[1..]).unwrap(),
        commit
    );
    assert!(CommitSmallAuctionSettlementV4Args::try_from(&bytes[..]).is_err());
    let keys = [1, 2, 3];
    let accounts = CommitSmallAuctionSettlementV4Accounts::try_from(keys.as_slice()).unwrap();
    assert_eq!(accounts.iter().copied().collect::<Vec<_>>(), keys);
    assert!(CommitSmallAuctionSettlementV4Accounts::try_from(&keys[..2]).is_err());
}
