use casper_types::{ContractHash, Key, SecretKey, U256};
use clap::Parser;
use verifier::VerifyKey;

#[derive(clap::Parser)]
enum Args {
    NFT {
        #[clap(short = 'k')]
        verification_key: String,
        #[clap(short = 'u', parse(try_from_str = parse_key))]
        user: Key,
        #[clap(short = 'c', parse(try_from_str = parse_key))]
        contract: Key,
        #[clap(short = 't')]
        token_id: u64,
        #[clap(short = 's')]
        signature: String,
    },
    Token {
        #[clap(short = 'k')]
        verification_key: String,
        #[clap(short = 'u', parse(try_from_str = parse_key))]
        user: Key,
        #[clap(short = 'c', parse(try_from_str = parse_key))]
        contract: Key,
        #[clap(short = 'v', parse(try_from_str = parse_u256))]
        value: U256,
        #[clap(short = 'n')]
        nonce: u64,
        #[clap(short = 's')]
        signature: String,
    },
}

fn parse_u256(arg: &str) -> Result<U256, anyhow::Error> {
    U256::from_dec_str(arg).map_err(|_| anyhow::anyhow!("failed to parse U256"))
}

fn parse_key(arg: &str) -> Result<Key, anyhow::Error> {
    Key::from_formatted_str(arg).map_err(|_| anyhow::anyhow!("failed to parse key"))
}

fn test_nft_signature(
    verification_key: String,
    user: Key,
    contract: Key,
    token_id: u64,
    signature: String,
) -> Result<(), anyhow::Error> {
    let verifier = VerifyKey::new(verification_key);
    verifier
        .verify_token_id(
            signature,
            token_id,
            ContractHash::from(contract.into_hash().unwrap()),
            user.into_account().unwrap().as_bytes(),
        )
        .unwrap();
    Ok(())
}

fn test_token_signature(
    verification_key: String,
    user: Key,
    contract: Key,
    value: U256,
    nonce: u64,
    signature: String,
) -> Result<(), anyhow::Error> {
    let verifier = VerifyKey::new(verification_key);
    verifier
        .verify_token_and_nonce(
            signature,
            value,
            nonce,
            ContractHash::from(contract.into_hash().unwrap()),
            user.into_account().unwrap().as_bytes(),
        )
        .unwrap();
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    match args {
        Args::NFT {
            verification_key,
            user,
            contract,
            token_id,
            signature,
        } => test_nft_signature(verification_key, user, contract, token_id, signature),
        Args::Token {
            verification_key,
            user,
            contract,
            value,
            nonce,
            signature,
        } => test_token_signature(verification_key, user, contract, value, nonce, signature),
    }?;

    println!("Signature is valid for given data");

    Ok(())
}
