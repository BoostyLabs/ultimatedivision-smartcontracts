use std::str::FromStr;

#[cfg(feature = "contract")]
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{
    bytesrepr::{Bytes, FromBytes},
    contracts::NamedKeys,
    ContractHash, Key, URef, U256,
};
use k256::ecdsa::{
    self,
    recoverable::{self, Signature},
    signature::{DigestSigner, Signature as _Signature, Signer},
    SigningKey, VerifyingKey,
};
use once_cell::unsync::OnceCell;
use tiny_keccak::Hasher;

#[cfg(feature = "contract")]
pub fn add_key(named_keys: &mut NamedKeys, verify_key: String) {
    VerifyKey::init(named_keys, verify_key)
}

#[cfg(feature = "contract")]
pub trait Verifier {
    fn verify_uuid(
        &self,
        signature: String,
        uuid: &str,
        contract: ContractHash,
        user: &[u8],
    ) -> Option<()> {
        VerifyKey::instance().verify_uuid(signature, uuid, contract, user)
    }

    fn verify_token_and_nonce(
        &self,
        signature: String,
        token_amount: U256,
        nonce: u64,
        contract: ContractHash,
        user: &[u8],
    ) -> Option<()> {
        VerifyKey::instance().verify_token_and_nonce(signature, token_amount, nonce, contract, user)
    }
}

#[cfg(feature = "contract")]
const VERIFY_KEY: &str = "verify_key";
const ETH_MESSAGE_PREFIX: &[u8] = b"\x19Ethereum Signed Message:\n32";

pub struct VerifyKey {
    key: OnceCell<Bytes>,
}

impl VerifyKey {
    #[cfg(feature = "contract")]
    fn instance() -> VerifyKey {
        let uref = runtime::get_key(VERIFY_KEY)
            .unwrap_or_revert()
            .into_uref()
            .unwrap_or_revert();
        let bytes = storage::read::<Bytes>(uref)
            .ok()
            .unwrap_or_revert()
            .unwrap_or_revert();

        VerifyKey {
            key: OnceCell::with_value(bytes),
        }
    }

    pub fn new(key: String) -> VerifyKey {
        let data = hex::decode(key).unwrap();
        let bytes = Bytes::from(data);
        VerifyKey {
            key: OnceCell::with_value(bytes),
        }
    }

    #[cfg(feature = "contract")]
    fn init(named_key: &mut NamedKeys, verify_key: String) {
        let bytes = hex::decode(verify_key).ok().unwrap_or_revert();
        let storage_uref = storage::new_uref(bytes).into_read();
        named_key.insert(VERIFY_KEY.to_string(), Key::from(storage_uref));
    }

    pub fn verify_uuid(
        &self,
        signature: String,
        token_id: &str,
        contract: ContractHash,
        user: &[u8],
    ) -> Option<()> {
        let mut bytes = vec![];
        bytes.extend_from_slice(user);
        bytes.extend_from_slice(contract.as_bytes());
        bytes.extend_from_slice(token_id.as_bytes());

        let result_hash = fake_ethereum_data_signing(keccak256(&bytes)); // Receive a hash of the packed data.

        let signature = cook_signature(signature)?;

        let key = signature
            .recover_verify_key_from_digest_bytes(result_hash.as_ref().into())
            .ok()?;

        if key == self.verifying_key()? {
            Some(())
        } else {
            None
        }
    }

    pub fn verify_token_and_nonce(
        &self,
        signature: String,
        token: U256,
        nonce: u64,
        contract: ContractHash,
        user: &[u8],
    ) -> Option<()> {
        let mut hex_token_string = format!("{:x}", &token);
        if hex_token_string.len() % 2 == 1 {
            hex_token_string.insert(0, '0');
        }
        let mut bytes = vec![];
        bytes.extend_from_slice(user);
        bytes.extend_from_slice(contract.as_bytes());
        bytes.extend(std::iter::repeat(0).take((64 - hex_token_string.len()) / 2)); // The number suppose to be 32 bytes
        bytes.extend_from_slice(&hex::decode(hex_token_string).ok()?);
        bytes.extend_from_slice(&[0; 24]); // The number should be 32 bytes
        bytes.extend_from_slice(&nonce.to_be_bytes());

        let result_hash = fake_ethereum_data_signing(keccak256(&bytes)); // Receive a hash of the packed data.

        let signature = cook_signature(signature)?;

        let key = signature
            .recover_verify_key_from_digest_bytes(result_hash.as_ref().into())
            .ok()?;

        if key == self.verifying_key()? {
            Some(())
        } else {
            None
        }
    }

    fn verifying_key(&self) -> Option<VerifyingKey> {
        VerifyingKey::from_sec1_bytes(self.key.get()?).ok()
    }
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);
    output
}

fn cook_signature(signature: String) -> Option<recoverable::Signature> {
    let mut signature = hex::decode(signature).ok()?;
    match signature[64] {
        // @see https://github.com/BoostyLabs/evmsignature/blob/master/signature.go Reform signature
        27 => signature[64] = 0,
        28 => signature[64] = 1,
        _ => {}
    };
    Signature::from_bytes(&signature).ok()
}

fn fake_ethereum_data_signing(hash: [u8; 32]) -> [u8; 32] {
    let mut data = ETH_MESSAGE_PREFIX.to_vec(); // Simulate ethereum signed message
    data.extend_from_slice(&hash);

    keccak256(&data)
}
