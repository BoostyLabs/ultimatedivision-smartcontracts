#![no_std]
#![no_main]

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{collections::BTreeMap, format, str, string::{String, ToString}, vec, vec::Vec};

use casper_contract::{
    contract_api::{
        runtime::{self, revert},
        storage, system,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, CLType, CLTyped, CLValue, ContractHash, ContractPackageHash, Key, Parameter,
    RuntimeArgs, URef, U128, U256, bytesrepr::ToBytes,
};

use contract_util::{current_contract, erc20};
use event::MarketEvent;
mod event;
use data::{
    contract_package_hash, emit, force_cancel_listing, get_id, get_listing, get_listing_dictionary,
    get_offers, get_purse, get_token_owner, token_id_to_vec, transfer_approved, Error, Listing,
};
mod data;
mod interface {
    pub mod onchain;
}

const OFFERS_PURSE: &str = "offers_purse";

const NFT_CONTRACT_HASH_ARG: &str = "nft_contract_hash";
const ERC20_CONTRACT_ARG: &str = "erc20_contract";
const TOKEN_ID_ARG: &str = "token_id";
const LISTING_ID_ARG: &str = "listing_id";
const MIN_BID_PRICE_ARG: &str = "min_bid_price";
const REDEMPTION_PRICE_ARG: &str = "redemption_price";
const AUCTION_DURATION_ARG: &str = "auction_duration";
const BUYER_PURSE_ARG: &str = "purse";
const ACCEPTED_OFFER_ARG: &str = "accepted_offer";

// vvvrev:
// +add min bid price
// +add redeem price
// +add check that redeem price >= redeem price
// +add auction_duration
// +check auction duration
// 2h + 2h tests = 4h
// transfer_nft? (+4h)
// cover: 50%
#[no_mangle]
pub extern "C" fn create_listing() -> () {
    let token_owner = Key::Account(runtime::get_caller());
    let nft_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let nft_contract_hash: ContractHash =
        ContractHash::from_formatted_str(&nft_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let min_bid_price: U256 = runtime::get_named_arg(MIN_BID_PRICE_ARG);
    let redemption_price: U256 = runtime::get_named_arg(REDEMPTION_PRICE_ARG);
    let auction_duration: U128 = runtime::get_named_arg(AUCTION_DURATION_ARG);

    if redemption_price.le(&min_bid_price) {
        runtime::revert(Error::RedemptionPriceLowerThanMinBid);
    }

    if auction_duration.is_zero() {
        runtime::revert(Error::AuctionDurationZero);
    }

    if token_owner != get_token_owner(nft_contract_hash, &token_id).unwrap() {
        runtime::revert(Error::PermissionDenied);
    }

    if !transfer_approved(nft_contract_hash, &token_id, token_owner) {
        runtime::revert(Error::NeedsTransferApproval);
    }

    let listing = Listing {
        nft_contract: nft_contract_hash,
        token_id: token_id.clone(),
        min_bid_price: min_bid_price,
        redemption_price: redemption_price,
        auction_duration: auction_duration,
        seller: token_owner,
    };

    let listing_id: String = get_id(&nft_contract_string, &token_id); // vvvcheck
    let dictionary_uref: URef = get_listing_dictionary();
    storage::dictionary_put(dictionary_uref, &listing_id, listing);

    emit(&MarketEvent::ListingCreated {
        package: contract_package_hash(),
        seller: token_owner,
        nft_contract: nft_contract_string,
        token_id: token_id,
        listing_id: listing_id,
        min_bid_price: min_bid_price,
        redemption_price: redemption_price,
        auction_duration: auction_duration,
    })
}

// vvvrev:
// add canceling offer +8h
// cover: 50%

#[no_mangle]
pub fn buy_listing() -> () {

    let buyer = Key::Account(runtime::get_caller());

    let nft_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let nft_contract_hash: ContractHash =
        ContractHash::from_formatted_str(&nft_contract_string).unwrap();

    let erc20_contract: ContractPackageHash = runtime::get_named_arg(ERC20_CONTRACT_ARG);

    let (self_contract_package, self_contract_hash) = current_contract();
    let self_contract_key: Key = (self_contract_package).into();

    let balance_before = erc20::balance_of(erc20_contract, buyer);

    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let token_ids: Vec<U256> = token_id_to_vec(&token_id);

    // vvvref: String->str?
    let listing_id: String = get_id(&nft_contract_string, &token_id);

    let listing: Listing = interface::onchain::get_listing_by_id(self_contract_hash, listing_id.clone());

    let text = format!(
        "VVV-buy_listing::nft_contract_string: {:?},\n\
         erc20_contract: {:?}\n\
         self_contract_key: {:?}\n\
         balance_before: {:?}\n\
         token_ids: {:?}\n\
         listing_redemption_price: {:?}\n\
         listing_seller: {:?}",
        nft_contract_string,
        erc20_contract,
        self_contract_key,
        balance_before,
        token_ids,
        listing.redemption_price,
        listing.seller
    );

    runtime::print(&text);


    erc20::transfer(erc20_contract, self_contract_key, listing.redemption_price);

    let balance_after = erc20::balance_of(erc20_contract, buyer);
    let text = format!(
        "VVV-buy_listing::balance_after: {:?},\n",
        balance_after
    );
    runtime::print(&text);

    let balance_after_seller = erc20::balance_of(erc20_contract, listing.seller);
    let text = format!(
        "VVV-buy_listing::balance_after_seller: {:?},\n",
        balance_after_seller
    );
    runtime::print(&text);

    // --------------------------------------------------------------------------------------------------------        

    // if balance_after.checked_sub(balance_before) != Some(listing.redemption_price) {
    //     revert(Error::UnexpectedTransferAmount)
    // }

    // vvvchange:
    // let buyer_purse: URef = runtime::get_named_arg(BUYER_PURSE_ARG); // vvvcheck
    // let purse_balance: U256 = system::get_purse_balance(buyer_purse).unwrap(); // vvvcheck
    // if purse_balance < listing.redemption_price {
    //     runtime::revert(Error::BalanceInsufficient);
    // }

    // --------------------------------------------------------------------------------------------------------        

    // runtime::call_contract::<()>(
    //     nft_contract_hash,
    //     "transfer_from",
    //     runtime_args! {
    //       "sender" => listing.seller,
    //       "recipient" => buyer,
    //       "token_ids" => token_ids,
    //     },
    // );
    // storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

    let text = format!(
        "VVV-buy_listing::nft_contract_string222: \n\
        package: {:?}\n\
        seller: {:?}\n\
        buyer: {:?}\n\
        nft_contract: {:?}\n\
        token_id: {:?}\n\
        min_bid_price: {:?}\n\
        redemption_price: {:?}\n\
        auction_duration: {:?}\n\
        ",
        contract_package_hash(),
        listing.seller,
        buyer,
        nft_contract_string,
        token_id,
        listing.min_bid_price,
        listing.redemption_price,
        listing.auction_duration
    );
    runtime::print(&text);

    let text = format!(
        "VVV-buy_listing::DATA {:?}
        ",
        listing
    );
    runtime::print(&text);
    interface::onchain::buy_listing_confirm(self_contract_hash, listing_id.clone(), buyer);
}

#[no_mangle]
pub fn get_listing_by_id() {
    let listing_id: String = runtime::get_named_arg("listing_id");
    
    let text = format!("VVV-get_listing_by_id {:?}", listing_id.to_string());
    runtime::print(&text);

    let (_listing, dictionary_uref) = get_listing(&listing_id);

    let text = format!("VVV-get_listing_by_id {:?}", _listing);
    runtime::print(&text);

    runtime::ret(CLValue::from_t(_listing).unwrap_or_revert());
}

#[no_mangle]
pub fn buy_listing_confirm() {
    let listing_id: String = runtime::get_named_arg("listing_id");
    let buyer: Key = runtime::get_named_arg("buyer");
    
    let (_listing, dictionary_uref) = get_listing(&listing_id);



    emit(&MarketEvent::ListingPurchased {
        package: contract_package_hash(),
        seller: _listing.seller,
        buyer: buyer,
        nft_contract: _listing.nft_contract.to_formatted_string(),
        token_id: _listing.token_id,
        min_bid_price: _listing.min_bid_price,
        redemption_price: _listing.redemption_price,
        auction_duration: _listing.auction_duration
    });



    // let text = format!("VVV-buy_listing_confirm {:?}", &_listing);
    // runtime::print(&text);
    let text = format!("VVV-buy_listing_confirm2 {:?}", buyer);
    runtime::print(&text);
    // let text = format!("VVV-buy_listing_confirm2 {:?}", Key::from_formatted_str(&buyer));
    // runtime::print(&text);

}

// vvvrev: do we need it?
#[no_mangle]
pub fn cancel_listing() -> () {
    let caller = Key::Account(runtime::get_caller());
    let nft_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let nft_contract_hash: ContractHash =
        ContractHash::from_formatted_str(&nft_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let listing_id: String = get_id(&nft_contract_string, &token_id);
    let seller = get_token_owner(nft_contract_hash, &token_id).unwrap();

    if caller != seller {
        runtime::revert(Error::PermissionDenied);
    }

    let (_listing, dictionary_uref) = get_listing(&listing_id);
    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

    emit(&MarketEvent::ListingCanceled {
        package: contract_package_hash(),
        nft_contract: nft_contract_string,
        token_id: token_id,
    })
}

// vvvrev:
// get previous offer, check and return it
// 8h-16h
// cover: 50%
#[no_mangle]
pub extern "C" fn make_offer() -> () {
    let bidder = Key::Account(runtime::get_caller());
    let nft_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let offers_id: String = get_id(&nft_contract_string, &token_id);
    let bidder_purse: URef = runtime::get_named_arg(BUYER_PURSE_ARG);
    let purse_balance: U256 = U256::one(); // system::get_purse_balance(bidder_purse).unwrap();

    let (mut offers, dictionary_uref): (BTreeMap<Key, U256>, URef) = get_offers(&offers_id);

    let offers_purse = get_purse(OFFERS_PURSE);

    // TODO: rebalance current offer instead of error
    match offers.get(&bidder) {
        Some(_) => runtime::revert(Error::OfferExists),
        None => (),
    }

    offers.insert(bidder, purse_balance);
    //system::transfer_from_purse_to_purse(bidder_purse, offers_purse, purse_balance, None).unwrap_or_revert();
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferCreated {
        package: contract_package_hash(),
        buyer: bidder,
        nft_contract: nft_contract_string,
        token_id: token_id,
        price: purse_balance,
    })
}

// vvvrev: re-use some code
#[no_mangle]
pub extern "C" fn withdraw_offer() -> () {
    let bidder = Key::Account(runtime::get_caller());
    let nft_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);

    let offers_id: String = get_id(&nft_contract_string, &token_id);

    let (mut offers, dictionary_uref): (BTreeMap<Key, U256>, URef) = get_offers(&offers_id);

    let amount: U256 = offers
        .get(&bidder)
        .unwrap_or_revert_with(Error::NoMatchingOffer)
        .clone();

    let offers_purse = get_purse(OFFERS_PURSE);

    // system::transfer_from_purse_to_account(
    //     offers_purse,
    //     bidder.into_account().unwrap_or_revert(),
    //     amount.clone(),
    //     None
    // ).unwrap_or_revert();

    offers.remove(&bidder);
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferWithdraw {
        package: contract_package_hash(),
        buyer: bidder,
        nft_contract: nft_contract_string,
        token_id: token_id,
    })
}

// vvvrev:
// reuse code
// 16h
// cover: 50%
#[no_mangle]
pub extern "C" fn accept_offer() -> () {
    let seller = Key::Account(runtime::get_caller());
    let nft_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let nft_contract_hash: ContractHash =
        ContractHash::from_formatted_str(&nft_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let token_ids: Vec<U256> = token_id_to_vec(&token_id);
    let offer_account_hash: String = runtime::get_named_arg(ACCEPTED_OFFER_ARG);
    let accepted_bidder_hash: Key = Key::from_formatted_str(&offer_account_hash).unwrap();
    let offers_id: String = get_id(&nft_contract_string, &token_id);
    let offers_purse = get_purse(OFFERS_PURSE);

    let (mut offers, dictionary_uref): (BTreeMap<Key, U256>, URef) = get_offers(&offers_id);

    let amount: U256 = offers
        .get(&accepted_bidder_hash)
        .unwrap_or_revert_with(Error::NoMatchingOffer)
        .clone();

    // system::transfer_from_purse_to_account(
    //     offers_purse,
    //     seller.into_account().unwrap_or_revert(),
    //     amount.clone(),
    //     None
    // ).unwrap_or_revert();
    offers.remove(&accepted_bidder_hash);

    runtime::call_contract::<()>(
        nft_contract_hash,
        "transfer_from",
        runtime_args! {
          "sender" => seller,
          "recipient" => accepted_bidder_hash,
          "token_ids" => token_ids,
        },
    );

    // refund the other offers
    // for (account, bid) in &offers {
    //     system::transfer_from_purse_to_account(
    //         offers_purse,
    //         account.into_account().unwrap_or_revert(),
    //         bid.clone(),
    //         None
    //     ).unwrap_or_revert();
    // }

    offers.clear();
    force_cancel_listing(&nft_contract_string, &token_id);
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferAccepted {
        package: contract_package_hash(),
        seller: seller,
        buyer: accepted_bidder_hash,
        nft_contract: nft_contract_string,
        token_id: token_id,
        price: amount,
    })
}

#[no_mangle]
pub extern "C" fn call() {
    let (contract_package_hash, _) = storage::create_contract_package_at_hash();
    let (contract_hash, _) = storage::add_contract_version(
        contract_package_hash,
        get_entry_points(),
        Default::default(),
    );
    runtime::put_key("market_contract_hash", contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key("market_contract_hash_wrapped", contract_hash_pack.into());
    runtime::put_key("market_contract_package_hash", contract_package_hash.into());
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "create_listing",
        vec![
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(MIN_BID_PRICE_ARG, U256::cl_type()),
            Parameter::new(REDEMPTION_PRICE_ARG, U256::cl_type()),
            Parameter::new(AUCTION_DURATION_ARG, U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "buy_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(ERC20_CONTRACT_ARG, ContractPackageHash::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(BUYER_PURSE_ARG, URef::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "cancel_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "make_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(BUYER_PURSE_ARG, URef::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "withdraw_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "accept_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(ACCEPTED_OFFER_ARG, URef::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_listing_by_id",
        vec![
            Parameter::new("listing_id", String::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "buy_listing_confirm",
        vec![
            Parameter::new("listing_id", String::cl_type()),
            Parameter::new("buyer", Key::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));


    entry_points
}
