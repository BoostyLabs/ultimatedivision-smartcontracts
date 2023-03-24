use alloc::{
    string::String
};
use casper_types::{ContractPackageHash, Key, U512, U128};

pub enum MarketEvent {
    ListingCreated {
        package: ContractPackageHash,
        seller: Key, //Key vs AccountHash so we know what we're getting client side
        token_contract: String,
        token_id: String,
        listing_id: String,
        min_bid_price: U512,
        redemption_price: U512,
        auction_duration: U128,
    },
    ListingPurchased {
        package: ContractPackageHash,
        seller: Key,
        buyer: Key,
        token_contract: String,
        token_id: String,
        min_bid_price: U512,
        redemption_price: U512,
        auction_duration: U128,
    },
    ListingCanceled {
        package: ContractPackageHash,
        token_contract: String,
        token_id: String
    },
    OfferCreated {
        package: ContractPackageHash,
        buyer: Key,
        token_contract: String,
        token_id: String,
        price: U512
    },
    OfferWithdraw {
        package: ContractPackageHash,
        buyer: Key,
        token_contract: String,
        token_id: String
    },
    OfferAccepted {
        package: ContractPackageHash,
        seller: Key,
        buyer: Key,
        token_contract: String,
        token_id: String,
        price: U512
    },
}