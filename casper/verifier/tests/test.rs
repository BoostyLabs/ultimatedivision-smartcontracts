use casper_types::{ContractHash, Key, U256};
use verifier::VerifyKey;

const KEY: &str = "02f083a879b53f9c013425dfeaa804731300dc37473940908c2150bc6a2243a548";

fn test_nft_signature(user: Key, contract: Key, token_id: String, signature: String) -> bool {
    let verifier = VerifyKey::new(KEY.to_string());
    verifier
        .verify_uuid(
            signature,
            &token_id,
            ContractHash::from(contract.into_hash().unwrap()),
            user.into_account().unwrap().as_bytes(),
        )
        .is_some()
}

fn test_token_signature(
    user: Key,
    contract: Key,
    nonce: u64,
    amount: U256,
    signature: String,
) -> bool {
    let verifier = VerifyKey::new(KEY.to_string());
    verifier
        .verify_token_and_nonce(
            signature,
            amount,
            nonce,
            ContractHash::from(contract.into_hash().unwrap()),
            user.into_account().unwrap().as_bytes(),
        )
        .is_some()
}

#[test]
fn nft_valid_signature() {
    let account = Key::from_formatted_str(
        "account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e",
    )
    .unwrap();
    let contract = Key::from_formatted_str(
        "hash-c3d2bdedf7f309e2908ecf90d0dfb44acf2a8077cf053a05779fb15bbbfdbfb9",
    )
    .unwrap();
    let signature = "ee64aa00a97ba58f17eb0aadd93b6b70e650b4b634080ba2dd9621e96e99ef83710f623d0338c3f24e9e0e4ed48e67d3b788e20c5e9feffb92623c03f88174d81b".to_string();
    let valid_uuid = "94b94d50-d001-4f88-b7cf-1763b39044b1".to_string();
    assert_eq!(
        test_nft_signature(account, contract, valid_uuid, signature.clone()),
        true
    );
    let invalid_uuid = "12312312".to_string();
    assert_eq!(
        test_nft_signature(account, contract, invalid_uuid, signature),
        false
    ); // Wrong token id
}

#[test]
fn nft_one_more_success() {
    let account = Key::from_formatted_str(
        "account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e",
    )
    .unwrap();
    let contract = Key::from_formatted_str(
        "hash-138c290930065b6f5a119f76e7b43df80fd6e01b0bb4737ac12866951b51497b",
    )
    .unwrap();
    let valid_uuid = "94b94d50-d001-4f88-b7cf-1763b39044b1".to_string();
    let signature = "a9b5f0287d5db7081f87add0aca5f638d662d9abe263d9970fa31d6e2ca0545d5e65c6a8c3306e2f41a2c0f73065a464aa079a40da2dc6c3f91dc19d35d2229b1c".to_string();

    assert_eq!(
        test_nft_signature(account, contract, valid_uuid, signature),
        true
    );
}

// #[test]
// fn token_valid_signature() {dbg!(m_formatted_str(
//         "hash-e8a213277d9c4ef1b9b6b3a0fcf0dac1a0a42dd009fd30ae899df5f9f1b88833",
//     )
//     .unwrap();
//     let amount = U256::from_dec_str("29979232443242342").unwrap();
//     let signature = "ee9a92cda137103fe47cfd0aceeada55b26b5253d78193ef6ac2361f1d5c9c562907681712cfb1fdea2830546681c447a82e5e7fba34b44cce60abe390d13fe21b".to_string();
//     assert_eq!(
//         test_token_signature(account, contract, 4, amount, signature.clone()),
//         true
//     );
//     assert_eq!(
//         test_token_signature(account, contract, 5, amount, signature.clone()),
//         false
//     );
//     assert_eq!(
//         test_token_signature(
//             account,
//             contract,
//             4,
//             U256::from_dec_str("32").unwrap(),
//             signature
//         ),
//         false
//     );
// }

#[test]
fn token_contract_test_signature() {
    let account = Key::from_formatted_str(
        "account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e",
    )
    .unwrap();
    let contract = Key::from_formatted_str(
        "hash-5aed0843516b06e4cbf56b1085c4af37035f2c9c1f18d7b0ffd7bbe96f91a3e0",
    )
    .unwrap();
    let amount = U256::from_dec_str("5000").unwrap();
    let signature = "a3f92029dae8b7a1fd682784995bd2fd3a395fe408c4eef6cccc358e7981b728625a6bb0a3bb2d91c4355ee7054bf9a2eef3aa8b31d63275eee02202d77a146a1b".to_string();
    assert_eq!(
        test_token_signature(account, contract, 0, amount, signature.clone()),
        true
    );
    assert_eq!(
        test_token_signature(account, contract, 5, amount, signature.clone()),
        false
    );
    assert_eq!(
        test_token_signature(
            account,
            contract,
            0,
            U256::from_dec_str("32").unwrap(),
            signature
        ),
        false
    );
}

#[test]
fn token_one_more_valid_signature() {
    let account = Key::from_formatted_str(
        "account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e",
    )
    .unwrap();
    let contract = Key::from_formatted_str(
        "hash-e8a213277d9c4ef1b9b6b3a0fcf0dac1a0a42dd009fd30ae899df5f9f1b88833",
    )
    .unwrap();
    let amount = U256::from_dec_str("299792").unwrap();
    let signature = "35e7807e6f3e1d161e34cf51f1d3d2c419e82e2ce48ce48bd38023983a689fbe30827aeaa4bdc18752994d08fe4569bf143a37149a9475e7e55e36726f9472c41b".to_string();
    assert_eq!(
        test_token_signature(account, contract, 1, amount, signature.clone()),
        true
    );
    assert_eq!(
        test_token_signature(account, contract, 5, amount, signature.clone()),
        false
    );
    assert_eq!(
        test_token_signature(
            account,
            contract,
            1,
            U256::from_dec_str("32").unwrap(),
            signature
        ),
        false
    );
}
