use alloc::{format, string::String, vec::Vec};
use casper_contract::{
    contract_api::runtime::{self, call_contract},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, CLType, CLValue, ContractHash, RuntimeArgs, U256};

use crate::data::Listing;

pub fn get_listing_by_id(market_contract: ContractHash, listing_id: String) -> bool {
    let data: String = call_contract::<String>(
        market_contract,
        "get_listing_by_id",
        runtime_args! {
          "listing_id" => listing_id,
        },
    );

    let text: String = format!("VVV::onchain::get_listing_by_id {:?}", data);
    runtime::print(&text);

    return true;
}
