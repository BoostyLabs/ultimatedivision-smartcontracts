use std::str::FromStr;

use casper_node::crypto::AsymmetricKeyExt;
use casper_types::{
    bytesrepr::{self, ToBytes},
    ContractHash, Key, SecretKey, U256, U512,
};
use k256::ecdsa::{recoverable, signature::Signer};

use clap::Parser;
use sha3::{digest::Update, Digest, Keccak256};

#[derive(clap::Parser)]
struct Args {
    #[clap(short = 'k')]
    private_key: String,
    #[clap(short = 'a')]
    amount: String,
    #[clap(short = 'n')]
    nonce: String,
    #[clap(short = 'c', parse(try_from_str = parse_key))]
    contract: Key,
    #[clap(short = 'u', parse(try_from_str = parse_key))]
    user: Key,
}

fn parse_key(arg: &str) -> Result<Key, anyhow::Error> {
    Key::from_formatted_str(arg).map_err(|_| anyhow::anyhow!("failed to parse key"))
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    println!("Loading key");

    let secret_key = casper_types::SecretKey::from_file(args.private_key).unwrap();
    let secret_key = if let SecretKey::Secp256k1(key) = secret_key {
        key
    } else {
        unreachable!();
    };

    println!("Creating verifying key");

    let verifying_key = secret_key.verify_key();
    println!("Creating digest");

    let digest = Keccak256::new()
        .chain(args.user.into_account().unwrap().as_bytes())
        .chain(ContractHash::from(args.contract.into_hash().unwrap()))
        .chain(
            U256::from_dec_str(&args.amount)
                .unwrap()
                .into_bytes()
                .unwrap(),
        )
        .chain(
            U256::from_dec_str(&args.nonce)
                .unwrap()
                .into_bytes()
                .unwrap(),
        )
        .finalize();
    println!("Signing message");
    let signature: recoverable::Signature = secret_key.sign(&digest);

    println!(
        "User hex key: {:?}",
        hex::encode(args.user.into_account().unwrap().as_bytes())
    );
    println!("Verifying key: {:?}", hex::encode(verifying_key.to_bytes()));
    println!("Signature: {:?}", hex::encode(signature));

    Ok(())
}
