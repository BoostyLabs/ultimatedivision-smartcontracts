#![no_std]
#![no_main]

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{
    borrow::ToOwned,
    str,
    string::{String},
    vec,
    vec::Vec,
};

use casper_contract::{
    contract_api::{
        runtime::{self},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    runtime_args,
    CLType, CLTyped, CLValue, ContractHash, Key, Parameter, RuntimeArgs, URef,
    U128, U256, Group,
};

use contract_util::{current_contract, erc20};
use event::MarketEvent;
mod event;
mod uref;
use data::{
    contract_package_hash, emit, remove_listing, get_id, get_listing, get_listing_dictionary,
    get_token_owner, token_id_to_vec, transfer_approved, AuctionBid,
    Error, Listing, get_initial_args, transfer_nft,
};
mod data;
mod interface {
    pub mod onchain;
}

// vvvref: move out constants
const NFT_CONTRACT_HASH_ARG: &str = "nft_contract_hash";
const AUCTION_DEFAULT_DURATION: u16 = 10800;
const TOKEN_ID_ARG: &str = "token_id";
const MIN_BID_PRICE_ARG: &str = "min_bid_price";
const OFFER_PRICE_ARG: &str = "offer_price";
const REDEMPTION_PRICE_ARG: &str = "redemption_price";
const AUCTION_DURATION_ARG: &str = "auction_duration";
const BUYER_PURSE_ARG: &str = "purse";

const GROUP_OPERATOR: &str = "operator";
const NK_ACCESS_UREF: &str = "market_contract_uref";

const PARAM_ERC20: &str = "erc20_hash";
const PARAM_STABLE_COMMISSION_PERCENT: &str = "stable_commission_percent";

// vvvunused OFFER functionality
// const ACCEPTED_OFFER_ARG: &str = "accepted_offer";

// vvvdone:
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
    let (token_owner, nft_contract_string, nft_contract_hash, token_id) = get_initial_args();

    let min_bid_price: U256 = runtime::get_named_arg(MIN_BID_PRICE_ARG);
    let redemption_price: U256 = runtime::get_named_arg(REDEMPTION_PRICE_ARG);
    let auction_duration: U128 = runtime::get_named_arg(AUCTION_DURATION_ARG);
    let current_time: u64 = runtime::get_blocktime().into();

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
        active_bid: None,
        created_time: U256::one() * current_time,
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
        created_time: U256::one() * current_time,
    })
}

// vvvdone:
// add canceling previous_offer_price if exists and transfer back money to the previous bidder +8h
// cover: 50%

#[no_mangle]
pub fn buy_listing() -> () {

    let (buyer, nft_contract_string, _, token_id) = get_initial_args();

    let erc20_hash: ContractHash = uref::read(PARAM_ERC20);

    let listing_id: &str = &(get_id(&nft_contract_string, &token_id).to_owned())[..];
    let (_listing, dictionary_uref) = get_listing(&listing_id);

    let buyer_balance = erc20::balance_of(erc20_hash, buyer);

    if buyer_balance < _listing.redemption_price {
        runtime::revert(Error::BalanceInsufficient);
    }

    erc20::transfer_from(
        erc20_hash,
        buyer,
        _listing.seller,
        _listing.redemption_price,
    );

    match _listing.active_bid {
        Some(bid) => {
            erc20::transfer_contract_to_recipient(erc20_hash, bid.bidder, bid.price);
        }
        None => (),
    }

    let token_ids: Vec<U256> = token_id_to_vec(&_listing.token_id);

    transfer_nft(_listing.nft_contract, _listing.seller, buyer, token_ids);

    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

    emit(&MarketEvent::ListingPurchased {
        package: contract_package_hash(),
        seller: _listing.seller,
        buyer: buyer,
        nft_contract: _listing.nft_contract.to_formatted_string(),
        token_id: _listing.token_id,
        min_bid_price: _listing.min_bid_price,
        redemption_price: _listing.redemption_price,
        auction_duration: _listing.auction_duration,
    });
}

#[no_mangle]
pub fn get_listing_by_id() {

    let listing_id: String = runtime::get_named_arg("listing_id");
    let (_listing, _) = get_listing(&listing_id);

    runtime::ret(CLValue::from_t(_listing).unwrap_or_revert());
}

// vvvdone:
// +transfer money to contract
// +remove previous_offer_price: transfer money back to the &bidder
// +check whether new bid greater than previous
// 4h
#[no_mangle]
pub extern "C" fn make_offer() -> () {
    let (bidder, nft_contract_string, _, token_id) = get_initial_args();

    let offer_price: U256 = runtime::get_named_arg(OFFER_PRICE_ARG);

    let erc20_hash: ContractHash = uref::read(PARAM_ERC20);

    let listing_id: &str = &(get_id(&nft_contract_string, &token_id).to_owned())[..];
    let (mut _listing, _) = get_listing(&listing_id);
    let (self_contract_package, _) = current_contract();
    let self_contract_key: Key = self_contract_package.into();

    if offer_price < _listing.min_bid_price {
        runtime::revert(Error::OfferPriceLessThanMinBid);
    }

    match _listing.active_bid {
        Some(bid) => {
            if offer_price <= bid.price {
                runtime::revert(Error::OfferPriceShouldBeGreaterThanPrevOffer);
            }
            erc20::transfer_contract_to_recipient(erc20_hash, bid.bidder, bid.price);
        }
        None => (),
    }

    _listing.active_bid = Some(AuctionBid {
        bidder: bidder,
        price: offer_price,
    });
    erc20::transfer_from(erc20_hash, bidder, self_contract_key, offer_price);

    let listing_id: String = get_id(&nft_contract_string, &token_id); // vvvcheck
    let listing_dictionary_uref: URef = get_listing_dictionary();
    storage::dictionary_put(listing_dictionary_uref, &listing_id, _listing);

    emit(&MarketEvent::OfferCreated {
        package: contract_package_hash(),
        buyer: bidder,
        nft_contract: nft_contract_string,
        token_id: token_id,
        price: offer_price,
    })
}

// vvvunused:
// #[no_mangle]
// pub extern "C" fn withdraw_offer() -> () {

//     let (bidder, nft_contract_string, _, token_id) = get_initial_args();

//     //remove_offer(&nft_contract_string, &token_id, &bidder);
//     // system::transfer_from_purse_to_account(
//     //     offers_purse,
//     //     bidder.into_account().unwrap_or_revert(),
//     //     amount.clone(),
//     //     None
//     // ).unwrap_or_revert();

//     emit(&MarketEvent::OfferWithdraw {
//         package: contract_package_hash(),
//         buyer: bidder,
//         nft_contract: nft_contract_string,
//         token_id: token_id,
//     })
// }

// vvvdone:
// reuse code
// 16h
// cover: 50%
#[no_mangle]
pub extern "C" fn accept_offer() -> () {
    let (seller, nft_contract_string, nft_contract_hash, token_id) = get_initial_args();
    let token_ids: Vec<U256> = token_id_to_vec(&token_id);
    let listing_id: &str = &(get_id(&nft_contract_string, &token_id).to_owned())[..];
    let (mut _listing, _) = get_listing(&listing_id);

    let erc20_hash: ContractHash = uref::read(PARAM_ERC20);

    match _listing.active_bid {
        Some(bid) => {

            if _listing.seller != seller {
                runtime::revert(Error::OfferPermissionDenied);
            }
            transfer_nft(nft_contract_hash, seller, bid.bidder, token_ids);
            erc20::transfer_contract_to_recipient(erc20_hash, seller, bid.price);

            remove_listing(&nft_contract_string, &token_id);
            emit(&MarketEvent::OfferAccepted {
                package: contract_package_hash(),
                seller: seller,
                buyer: bid.bidder,
                nft_contract: nft_contract_string,
                token_id: token_id,
                price: bid.price,
            });
        }
        None => runtime::revert(Error::OfferNotFound),
    }
}

#[no_mangle]
pub extern "C" fn final_listing() -> () {
    let (_, nft_contract_string, nft_contract_hash, token_id) = get_initial_args();

    let token_ids: Vec<U256> = token_id_to_vec(&token_id);

    let listing_id: &str = &(get_id(&nft_contract_string, &token_id).to_owned())[..];
    let (mut _listing, _) = get_listing(&listing_id);

    let erc20_hash: ContractHash = uref::read(PARAM_ERC20);

    let current_time: u64 = runtime::get_blocktime().into();
    

    let listing_finish_time = U128::as_u64(&_listing.auction_duration) + U256::as_u64(&_listing.created_time); 

    let text = &alloc::format!("listing_finish_time {:?}", {listing_finish_time});
    runtime::print(&text);
    let text = &alloc::format!("current_time {:?}", current_time);
    runtime::print(&text);

    if current_time < listing_finish_time {
        // vvv: uncomment!
        // runtime::revert(Error::ListingTimeNotFinished);
    }

    remove_listing(&nft_contract_string, &token_id);

    match _listing.active_bid {
        Some(bid) => {
            transfer_nft(nft_contract_hash, _listing.seller, bid.bidder, token_ids);
            erc20::transfer_contract_to_recipient(erc20_hash, _listing.seller, bid.price);
            emit(&MarketEvent::OfferAccepted {
                package: contract_package_hash(),
                seller: _listing.seller,
                buyer: bid.bidder,
                nft_contract: nft_contract_string,
                token_id: token_id,
                price: bid.price,
            });
        }
        None => {
            emit(&MarketEvent::ListingFinishedWithoutOffer {
                package: contract_package_hash(),
                seller: _listing.seller,
                nft_contract: nft_contract_string,
                token_id: token_id,
            });
        }
    };



}

#[no_mangle]
pub extern "C" fn call() {

    let (contract_package_hash, access_uref) = storage::create_contract_package_at_hash();
    let mut named_keys = NamedKeys::new();

    let erc20_hash: ContractHash = runtime::get_named_arg(PARAM_ERC20);
    // let text = &alloc::format!("VVV-erc20 {:?}", erc20_hash);
    // runtime::print(&text);

    let default_percent = storage::new_uref(U256::one() * 3);
    let default_erc20 = storage::new_uref(erc20_hash);

    let default_percent_key_name = String::from(PARAM_STABLE_COMMISSION_PERCENT);
    named_keys.insert(default_percent_key_name, Key::URef(default_percent));

    let signer_key = String::from(PARAM_ERC20);
    named_keys.insert(signer_key, Key::URef(default_erc20));

    storage::create_contract_user_group(
        contract_package_hash, 
        GROUP_OPERATOR,
        0,
        [access_uref].into()
    )
    .unwrap_or_revert();

    runtime::put_key(NK_ACCESS_UREF, access_uref.into());

    let (contract_hash, _) = storage::add_contract_version(
        contract_package_hash,
        get_entry_points(),
        named_keys,
    );
    runtime::put_key("market_contract_hash", contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key("market_contract_hash_wrapped", contract_hash_pack.into());
    runtime::put_key("market_contract_package_hash", contract_package_hash.into());
}
fn operator_access() -> EntryPointAccess {
    EntryPointAccess::Groups(vec![Group::new(GROUP_OPERATOR)])
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "create_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
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
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(BUYER_PURSE_ARG, URef::cl_type()),
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
            Parameter::new(OFFER_PRICE_ARG, U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    // entry_points.add_entry_point(EntryPoint::new(
    //     "withdraw_offer",
    //     vec![
    //         Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
    //         Parameter::new(TOKEN_ID_ARG, String::cl_type()),
    //     ],
    //     <()>::cl_type(),
    //     EntryPointAccess::Public,
    //     EntryPointType::Contract,
    // ));
    entry_points.add_entry_point(EntryPoint::new(
        "accept_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "final_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
        ],
        <()>::cl_type(),
        operator_access(),
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_listing_by_id",
        vec![Parameter::new("listing_id", String::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}

// vvvrev: add commission logic
// vvvrev: hardcode erc20 contract?
// vvvrev: pass contract as bytes - how to?