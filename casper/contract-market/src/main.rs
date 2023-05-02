#![no_std]
#![no_main]

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{collections::BTreeMap, format, str, string::{String, ToString}, vec, vec::Vec, borrow::ToOwned};

use casper_contract::{
    contract_api::{
        runtime::{self, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, CLType, CLTyped, CLValue, ContractHash, ContractPackageHash, Key, Parameter,
    RuntimeArgs, URef, U128, U256,
};

use contract_util::{current_contract, erc20};
use event::MarketEvent;
mod event;
use data::{
    contract_package_hash, emit, force_cancel_listing, get_id, get_listing, get_listing_dictionary,
    get_offers, get_token_owner, token_id_to_vec, transfer_approved, Error, Listing,
};
mod data;
mod interface {
    pub mod onchain;
}

// vvvref: move out constants
const NFT_CONTRACT_HASH_ARG: &str = "nft_contract_hash";
const AUCTION_DEFAULT_DURATION: u16 = 10800;
const ERC20_CONTRACT_ARG: &str = "erc20_contract";
const TOKEN_ID_ARG: &str = "token_id";
const MIN_BID_PRICE_ARG: &str = "min_bid_price";
const OFFER_PRICE_ARG: &str = "offer_price";
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
// +transfer_nft? (+4h), done via approval
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

    if auction_duration.lt(&(U128::one() * AUCTION_DEFAULT_DURATION)) {
        runtime::revert(Error::AuctionInvalidDuration);
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

    let erc20_contract: ContractPackageHash = runtime::get_named_arg(ERC20_CONTRACT_ARG);

    let (_, self_contract_hash) = current_contract();

    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);

    // vvvref: String->str?
    let listing_id: &str = &(get_id(&nft_contract_string, &token_id).to_owned())[..];

    let listing = interface::onchain::get_listing_by_id(self_contract_hash, listing_id);

    let balance_before_seller = erc20::balance_of(erc20_contract, listing.seller);

    let buyer_balance = erc20::balance_of(erc20_contract, buyer);
    if buyer_balance < listing.redemption_price {
        runtime::revert(Error::BalanceInsufficient);
    }

    erc20::transfer(erc20_contract, listing.seller, listing.redemption_price);

    let balance_after_seller = erc20::balance_of(erc20_contract, listing.seller);

    if balance_after_seller.checked_sub(balance_before_seller) != Some(listing.redemption_price) {
        revert(Error::UnexpectedTransferAmount)
    }

    interface::onchain::buy_listing_confirm(self_contract_hash, listing_id, buyer);
}

#[no_mangle]
pub fn get_listing_by_id() {
    let listing_id: String = runtime::get_named_arg("listing_id");
    
    let (_listing, _) = get_listing(&listing_id);

    runtime::ret(CLValue::from_t(_listing).unwrap_or_revert());
}

#[no_mangle]
pub fn buy_listing_confirm() {
    let listing_id: String = runtime::get_named_arg("listing_id");
    let buyer: Key = runtime::get_named_arg("buyer");
    
    let (_listing, dictionary_uref) = get_listing(&listing_id);

    let token_ids: Vec<U256> = token_id_to_vec(&_listing.token_id);
    runtime::call_contract::<()>(
        _listing.nft_contract,
        "transfer_from",
        runtime_args! {
          "sender" => _listing.seller,
          "recipient" => buyer,
          "token_ids" => token_ids,
        },
    );
    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

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
    let offer_price: U256 = runtime::get_named_arg(OFFER_PRICE_ARG);

    let listing_id: &str = &(get_id(&nft_contract_string, &token_id).to_owned())[..];
    let (_listing, _) = get_listing(&listing_id);

    let text = format!("VVV-make_offer {:?}", _listing);
    runtime::print(&text);


    let (mut offers, dictionary_uref): (BTreeMap<Key, U256>, URef) = get_offers(&offers_id);

    if offer_price < _listing.min_bid_price {
        runtime::revert(Error::OfferPriceLessThanMinBid);
    }
    // TODO: rebalance current offer instead of error
    match offers.get(&bidder) {
        Some(_) => runtime::revert(Error::OfferExists),
        None => (),
    }

    offers.insert(bidder, offer_price);
    // vvvrev:
    //system::transfer_from_purse_to_purse(bidder_purse, offers_purse, purse_balance, None).unwrap_or_revert();
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferCreated {
        package: contract_package_hash(),
        buyer: bidder,
        nft_contract: nft_contract_string,
        token_id: token_id,
        price: offer_price,
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
        "make_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(OFFER_PRICE_ARG, U256::cl_type()),
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
