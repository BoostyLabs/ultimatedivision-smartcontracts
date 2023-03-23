#![allow(non_snake_case)]
use casper_types::{U256};
pub const TEST_BLOCK_TIME: u64 = 1672071121;
pub(crate) fn TEST_AMOUNT() -> U256 {
    U256::one() * 1_000_000_000_000u64
}


pub(crate) const TEST_ACCOUNT_BALANCE: u64 = 10_000_000_000_000u64;

pub(crate) const TEST_ACCOUNT: [u8; 32] = [255u8; 32];

pub const PARAM_RECIPIENT: &str = "recipient";
pub const PARAM_AMOUNT: &str = "amount";
pub const PARAM_NFT_NAME: &str = "TestDragonNFT";
pub const PARAM_NFT_SYMBOL: &str = "DGNFT";
pub const PARAM_NFT_CONTRACT_NAME: &str = "TestDragonNFT";
pub const PARAM_NFT_PRICE: u64 = 111;
pub const PARAM_MARKET_CONTRACT_NAME: &str = "market";

pub const EP_MINT: &str = "mint_copies";
pub const EP_CREATE_LISTING: &str = "create_listing";
pub const EP_APPROVE: &str = "approve";
