use casper_types::{ContractHash, Key, U256};
use verifier::VerifyKey;

const KEY: &str = "02f083a879b53f9c013425dfeaa804731300dc37473940908c2150bc6a2243a548";

fn test_nft_signature(user: Key, contract: Key, token_id: u64, signature: String) -> bool {
    let verifier = VerifyKey::new(KEY.to_string());
    verifier
        .verify_token_id(
            signature,
            token_id,
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
        "hash-e8a213277d9c4ef1b9b6b3a0fcf0dac1a0a42dd009fd30ae899df5f9f1b88833",
    )
    .unwrap();
    let signature = "fd111c49caf6960cd6e92c274af15d20d09d70cfd1f9b9126d9f0d3b183140883ca9aafebc3a27581b78f323dcf5b5381c1725dcc69839ce78c364b245ab068f1b".to_string();
    assert_eq!(
        test_nft_signature(account, contract, 6, signature.clone()),
        true
    );
    assert_eq!(test_nft_signature(account, contract, 4, signature), false); // Wrong token id
}

#[test]
fn nft_one_more_success() {
    let account = Key::from_formatted_str(
        "account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e",
    )
    .unwrap();
    let contract = Key::from_formatted_str(
        "hash-fced3ea436da29aa2715d6cb071d813801a0c63097bc75e0bdec907e37a69869",
    )
    .unwrap();
    let signature = "6b0c7da4353cfa0f00cea1fdbc9df04b6974bdfd9947ce8c9b5c3d26cd8190cd4a5590ab1b61e27c79b72e3dd717b63df333779580d4516d1d509deb425e9b701c".to_string();
    assert_eq!(test_nft_signature(account, contract, 4, signature), true);
}

#[test]
fn token_valid_signature() {
    let account = Key::from_formatted_str(
        "account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e",
    )
    .unwrap();
    let contract = Key::from_formatted_str(
        "hash-e8a213277d9c4ef1b9b6b3a0fcf0dac1a0a42dd009fd30ae899df5f9f1b88833",
    )
    .unwrap();
    let amount = U256::from_dec_str("29979232443242342").unwrap();
    let signature = "ee9a92cda137103fe47cfd0aceeada55b26b5253d78193ef6ac2361f1d5c9c562907681712cfb1fdea2830546681c447a82e5e7fba34b44cce60abe390d13fe21b".to_string();
    assert_eq!(
        test_token_signature(account, contract, 4, amount, signature.clone()),
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
            4,
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
