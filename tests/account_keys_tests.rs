use ambient_auction_api::instruction::*;

#[test]
fn init_bundle_account_keys_round_trip_and_order() {
    let keys = InitBundleAccountKeys {
        payer: 1_u8,
        bundle: 2,
        registry: 3,
        system_program: 4,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[test]
fn place_bid_account_keys_round_trip_and_order() {
    let keys = PlaceBidAccountKeys {
        payer: 1_u8,
        bid: 2,
        auction: 3,
        system_program: 4,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[test]
fn reveal_bid_account_keys_round_trip_and_order() {
    let keys = RevealBidAccountKeys {
        bid_authority: 1_u8,
        bid: 2,
        auction: 3,
        bundle: 4,
        vote_account: 5,
        vote_authority: 6,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[test]
fn submit_job_output_account_keys_round_trip_and_order() {
    let keys = SubmitJobOutputAccountKeys {
        bid_authority: 1_u8,
        bundle: 2,
        job_request: 3,
        bid: 4,
        auction: 5,
        output_data_account: 6,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[test]
fn open_bundle_escrow_v2_account_keys_round_trip_and_order() {
    let keys = OpenBundleEscrowV2AccountKeys {
        payer: 1_u8,
        bundle_escrow: 2,
        config_policy: 3,
        system_program: 4,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[cfg(feature = "global-config")]
#[test]
fn init_config_account_keys_round_trip_and_order() {
    let keys = InitConfigAccountKeys {
        payer: 1_u8,
        config: 2,
        system_program: 3,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[cfg(not(feature = "global-config"))]
#[test]
fn request_job_account_keys_round_trip_and_preserves_pair_order() {
    let keys = RequestJobAccountKeys {
        payer: 1_u8,
        job_request: 2,
        registry: 3,
        input_data: 4,
        system_program: 5,
        bundle_auction_account_pairs: vec![6, 7, 8, 9, 10, 11],
        last_bundle: 12,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}

#[cfg(feature = "global-config")]
#[test]
fn request_job_account_keys_round_trip_and_preserves_pair_order() {
    let keys = RequestJobAccountKeys {
        payer: 1_u8,
        job_request: 2,
        registry: 3,
        input_data: 4,
        system_program: 5,
        config: 6,
        bundle_auction_account_pairs: vec![7, 8, 9, 10, 11, 12],
        last_bundle: 13,
    };

    assert_eq!(
        keys.as_accounts().iter_owned().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
    );
    assert_eq!(keys.as_accounts().to_account_keys(), keys);
}
