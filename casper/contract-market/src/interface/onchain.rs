use alloc::{string::String, vec::Vec, format};
use casper_contract::{contract_api::runtime::{call_contract, self}, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{
    ContractHash, RuntimeArgs, U256, CLValue, CLType,
};

use crate::data::Listing;


pub fn get_listing_by_id(market_contract: ContractHash, listing_id: String) -> bool {

    let text = format!("VVV-buy_listing::nft_contract_string5 {:?}", market_contract);
    runtime::print(&text);

    let args = RuntimeArgs::try_new(|args| {
        args.insert("listing_id", U256::one())?;
        Ok(())
    }).unwrap_or_revert();

    let text = format!("VVV-buy_listing::nft_contract_string5___1");
    runtime::print(&text);

    call_contract::<()>(
        market_contract,
        "get_listing_by_id",
        args,
    );
    let text = format!("VVV-buy_listing::nft_contract_string7");
    runtime::print(&text);

    

    return true;
}
