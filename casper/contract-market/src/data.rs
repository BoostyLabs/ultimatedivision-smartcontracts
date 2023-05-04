use alloc::{
    string::{String, ToString},
    str,
    vec, vec::Vec,
    collections::BTreeMap, format
};

use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{
    system::CallStackElement,
    bytesrepr::ToBytes,
    runtime_args, RuntimeArgs,
    ApiError, Key, URef, ContractHash, ContractPackageHash, CLTyped, U256, U128};

use casper_types_derive::{CLTyped, FromBytes, ToBytes};

use crate::{
    event::MarketEvent
};

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
pub enum Error {
    ListingDoesNotExist = 1000,
    ListingCanceledOrSold = 1001,
    BalanceInsufficient = 1002,
    PermissionDenied = 1003,
    NoMatchingOffer = 1004,
    OfferExists = 1005,
    OfferPurseRetrieval = 1006,
    NeedsTransferApproval = 1007,
    RedemptionPriceLowerThanMinBid = 1008,
    AuctionInvalidDuration = 1009,
    UnexpectedTransferAmount = 1010,
    OfferPriceLessThanMinBid = 1011,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

// vvvfix: remove debug

// struct being used only for workaround to dictionary limitation (no remove function)
#[derive(CLTyped, ToBytes, FromBytes, Debug)]
pub struct Listing {
    pub seller: Key,
    pub nft_contract: ContractHash,
    pub token_id: String,
    pub min_bid_price: U256,
    pub redemption_price: U256,
    pub auction_duration: U128,
    // vvvrev: add params
}

const EVENT_TYPE: &str = "event_type";
const CONTRACT_PACKAGE_HASH: &str = "contract_package_hash";
const SELLER: &str = "seller";
const BUYER: &str = "buyer";
const NFT_CONTRACT: &str = "nft_contract";
const TOKEN_ID: &str = "token_id";
const LISTING_ID: &str = "listing_id";
const MIN_BID_PRICE: &str = "min_bid_price";
const REDEMPTION_PRICE: &str = "redemption_price";
const AUCTION_DURATION: &str = "auction_duration";

const LISTING_DICTIONARY: &str = "listings";
const OFFER_DICTIONARY: &str = "offers";

pub fn contract_package_hash() -> ContractPackageHash {
    let call_stacks = runtime::get_call_stack();



    let last_entry = call_stacks.last().unwrap_or_revert();


    let package_hash: Option<ContractPackageHash> = match last_entry {
        CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } => Some(*contract_package_hash),
        CallStackElement::StoredSession {
            contract_package_hash,
            contract_hash: _,
            account_hash: _
        } => Some(*contract_package_hash),
        _ => None,
    };
    let text = format!(
        "VVC: {:?}\n\
        ",
        last_entry
    );
    runtime::print(&text);

    package_hash.unwrap_or_revert()
}

pub fn transfer_approved(nft_contract_hash: ContractHash, token_id: &str, owner: Key) -> bool {
    let approved = runtime::call_contract::<Option<Key>>(
        nft_contract_hash,
        "get_approved",
        runtime_args! {
            "owner" => owner,
            "token_id" => U256::from_dec_str(&token_id).unwrap()
          }
    );

    contract_package_hash().value() == approved
            .unwrap_or_revert_with(Error::NeedsTransferApproval)
            .into_hash()
            .unwrap()
}

pub fn get_id<T: CLTyped + ToBytes>(nft_contract: &T, token_id: &T) -> String {
    let mut bytes_a = nft_contract.to_bytes().unwrap_or_revert();
    let mut bytes_b = token_id.to_bytes().unwrap_or_revert();

    bytes_a.append(&mut bytes_b);

    let bytes = runtime::blake2b(bytes_a);
    hex::encode(bytes)
}

pub fn get_dictionary_uref(key: &str) -> URef {
    match runtime::get_key(key) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(key).unwrap_or_revert(),
    }
}

pub fn get_token_owner(nft_contract_hash: ContractHash, token_id: &str) -> Option<Key> {
    runtime::call_contract::<Option<Key>>(
        nft_contract_hash,
        "owner_of",
        runtime_args! {
            "token_id" => U256::from_dec_str(&token_id).unwrap()
          }
    )
}

pub fn token_id_to_vec(token_id: &str) -> Vec<U256> {
    let token_id: U256 = U256::from_str_radix(&token_id, 10).unwrap();
    vec![token_id]
}

pub fn get_listing(listing_id: &str) -> (Listing, URef) {
    let dictionary_uref = get_dictionary_uref(LISTING_DICTIONARY);

    let listing : Listing =
        match storage::dictionary_get(dictionary_uref, &listing_id)  {
            Ok(item) => match item {
                None => runtime::revert(Error::ListingDoesNotExist),
                Some(value) => value,
            },
            Err(_error) => runtime::revert(Error::ListingCanceledOrSold)
        };

    (listing, dictionary_uref)
}

pub fn get_listing_dictionary() -> URef {
    get_dictionary_uref(LISTING_DICTIONARY)
}

// use when it doesn't matter if listing exists or not & no event needed
pub fn force_cancel_listing(nft_contract: &str, token_id: &str) -> () {
    let listing_id: String = get_id(&nft_contract, &token_id);
    let dictionary_uref = get_dictionary_uref(LISTING_DICTIONARY);
    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);
}

pub fn get_offers(offers_id: &str) -> (BTreeMap<Key, U256>, URef) {
    let dictionary_uref = get_dictionary_uref(OFFER_DICTIONARY);

    let offers: BTreeMap<Key, U256> =
        match storage::dictionary_get(dictionary_uref, &offers_id)  {
            Ok(item) => match item {
                None => BTreeMap::new(),
                Some(offers) => offers,
            },
            Err(_error) => BTreeMap::new()
        };

    return (offers, dictionary_uref);
}

pub fn get_purse(purse_name: &str) -> URef {
    let purse = if !runtime::has_key(&purse_name) {
        let purse = system::create_purse();
        runtime::put_key(&purse_name, purse.into());
        purse
    } else {
        let destination_purse_key = runtime::get_key(&purse_name).unwrap_or_revert_with(
            Error::OfferPurseRetrieval
        );
        match destination_purse_key.as_uref() {
            Some(uref) => *uref,
            None => runtime::revert(Error::OfferPurseRetrieval),
        }
    };
    return purse;
}

pub fn remove_offer(nft_contract_string: &String, token_id: &String, bidder: &Key) {
    let offers_id: String = get_id(nft_contract_string, token_id);
    let (mut offers, dictionary_uref): (BTreeMap<Key, U256>, URef) = get_offers(&offers_id);
    offers.remove(&bidder);
    storage::dictionary_put(dictionary_uref, &offers_id, offers);
    // vvvrev: transfer money back to the &bidder 
}

pub fn emit(event: &MarketEvent) {
    let push_event = match event {
        MarketEvent::ListingCreated {
            package,
            seller,
            nft_contract,
            token_id,
            listing_id,
            min_bid_price,
            redemption_price,
            auction_duration
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(SELLER, seller.to_string());
            param.insert(NFT_CONTRACT, nft_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(LISTING_ID, listing_id.to_string());
            param.insert(MIN_BID_PRICE, min_bid_price.to_string());
            param.insert(REDEMPTION_PRICE, redemption_price.to_string());
            param.insert(AUCTION_DURATION, auction_duration.to_string());
            param.insert(EVENT_TYPE, "market_listing_created".to_string());
            param
        }
        MarketEvent::ListingPurchased {
            package,
            seller,
            buyer,
            nft_contract,
            token_id,
            min_bid_price,
            redemption_price,
            auction_duration
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(SELLER, seller.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(NFT_CONTRACT, nft_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(MIN_BID_PRICE, min_bid_price.to_string());
            param.insert(REDEMPTION_PRICE, redemption_price.to_string());
            param.insert(AUCTION_DURATION, auction_duration.to_string());
            param.insert(EVENT_TYPE, "market_listing_purchased".to_string());
            param
        }
        MarketEvent::OfferCreated {
            package,
            buyer,
            nft_contract,
            token_id,
            price
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(NFT_CONTRACT, nft_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(REDEMPTION_PRICE, price.to_string());
            param.insert(EVENT_TYPE, "market_offer_created".to_string());
            param
        },
        MarketEvent::OfferWithdraw {
            package,
            buyer,
            nft_contract,
            token_id
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(NFT_CONTRACT, nft_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(EVENT_TYPE, "market_offer_withdraw".to_string());
            param
        },
        MarketEvent::OfferAccepted {
            package,
            seller,
            buyer,
            nft_contract,
            token_id,
            price
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(SELLER, seller.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(NFT_CONTRACT, nft_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(REDEMPTION_PRICE, price.to_string());
            param.insert(EVENT_TYPE, "market_offer_accepted".to_string());
            param
        }
    };
    let latest_event: URef = storage::new_uref(push_event);
    runtime::put_key("latest_event", latest_event.into());
}