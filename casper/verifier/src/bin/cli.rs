use casper_types::{ContractHash, Key, SecretKey};
use clap::Parser;
use verifier::VerifyKey;

#[derive(clap::Parser)]
enum Args {
    TestNFT {
        #[clap(short = 'k')]
        private_key: String,
        #[clap(short = 'u', parse(try_from_str = parse_key))]
        user: Key,
        #[clap(short = 'c', parse(try_from_str = parse_key))]
        contract: Key,
        #[clap(short = 't')]
        token_id: u64,
        #[clap(short = 's')]
        signature: String,
    },
}

fn parse_key(arg: &str) -> Result<Key, anyhow::Error> {
    Key::from_formatted_str(arg).map_err(|_| anyhow::anyhow!("failed to parse key"))
}

fn test_nft_signature(
    private_key: String,
    user: Key,
    contract: Key,
    token_id: u64,
    signature: String,
) -> Result<(), anyhow::Error> {
    let secret_key =
        casper_types::SecretKey::secp256k1_from_bytes(hex::decode(private_key).unwrap()).unwrap();
    let secret_key = if let SecretKey::Secp256k1(key) = secret_key {
        key
    } else {
        unreachable!();
    };
    let verifier = VerifyKey::new(hex::encode(secret_key.verify_key().to_bytes()));
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

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    match args {
        Args::TestNFT {
            private_key,
            user,
            contract,
            token_id,
            signature,
        } => test_nft_signature(private_key, user, contract, token_id, signature),
    }?;

    Ok(())
}
