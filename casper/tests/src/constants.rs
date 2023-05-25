#![allow(non_snake_case)]
pub const TEST_BLOCK_TIME: u64 = 1672071121;


pub(crate) const TEST_ACCOUNT_BALANCE: u64 = 10_000_000_000_000u64;

pub(crate) const TEST_ACCOUNT: [u8; 32] = [255u8; 32];

pub const PARAM_RECIPIENT: &str = "recipient";
pub const PARAM_AMOUNT: &str = "amount";
pub const PARAM_NFT_NAME: &str = "TestDragonNFT";
pub const PARAM_NFT_SYMBOL: &str = "DGNFT";
pub const PARAM_NFT_CONTRACT_NAME: &str = "TestDragonNFT";
pub const PARAM_NFT_PRICE: u64 = 111;
pub const PARAM_MARKET_CONTRACT_NAME: &str = "market";
pub const PARAM_COMMISSION_WALLET: &str = "commission_wallet";
pub const PARAM_STABLE_COMMISSION_PERCENT: &str = "stable_commission_percent";

pub const EP_MINT: &str = "mint_copies";
pub const EP_CREATE_LISTING: &str = "create_listing";
pub const EP_BUY_LISTING: &str = "buy_listing";
pub const EP_MAKE_OFFER: &str = "make_offer";
pub const EP_ACCEPT_OFFER: &str = "accept_offer";
pub const EP_FINAL_LISTING: &str = "final_listing";
pub const EP_APPROVE: &str = "approve";
pub const EP_SET_COMMISSION_WALLET: &str = "set_commission_wallet";
pub const EP_SET_STABLE_COMMISSION_PERCENT: &str = "set_stable_commission_percent";
