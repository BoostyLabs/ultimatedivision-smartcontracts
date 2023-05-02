use casper_contract::contract_api::runtime::{call_contract};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};

use crate::data::Listing;

// vvvq - whether we need to return tuple here?
pub fn get_listing_by_id(market_contract: ContractHash, listing_id: &str) -> Listing {
    call_contract::<Listing>(
        market_contract,
        "get_listing_by_id",
        runtime_args! {
          "listing_id" => listing_id,
        },
    )
}

pub fn buy_listing_confirm(market_contract: ContractHash, listing_id: &str, buyer: Key) {
    call_contract::<()>(
        market_contract,
        "buy_listing_confirm",
        runtime_args! {
          "listing_id" => listing_id,
          "buyer" => buyer
        },
    );
}
