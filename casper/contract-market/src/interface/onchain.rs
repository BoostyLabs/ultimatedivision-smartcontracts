use alloc::{format, string::{String, ToString}, vec::Vec};
use casper_contract::{
    contract_api::runtime::{self, call_contract},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, CLType, CLValue, ContractHash, RuntimeArgs, U256, Key};

use crate::data::Listing;

// vvvq - whether we need to return tuple here?
pub fn get_listing_by_id(market_contract: ContractHash, listing_id: String) -> Listing {
    call_contract::<Listing>(
        market_contract,
        "get_listing_by_id",
        runtime_args! {
          "listing_id" => listing_id,
        },
    )
}

pub fn buy_listing_confirm(market_contract: ContractHash, listing_id: String, buyer: Key) {
    call_contract::<()>(
        market_contract,
        "buy_listing_confirm",
        runtime_args! {
          "listing_id" => listing_id,
          "buyer" => buyer
        },
    );
}
