use alloc::{
    string::String
};
use casper_types::{ContractPackageHash, Key, U128, U256};

pub enum MarketEvent {
    ListingCreated {
        package: ContractPackageHash,
        seller: Key, //Key vs AccountHash so we know what we're getting client side
        nft_contract: String,
        token_id: String,
        listing_id: String,
        min_bid_price: U256,
        redemption_price: U256,
        auction_duration: U128,
    },
    ListingPurchased {
        package: ContractPackageHash,
        seller: Key,
        buyer: Key,
        nft_contract: String,
        token_id: String,
        min_bid_price: U256,
        redemption_price: U256,
        auction_duration: U128,
    },
    ListingCanceled {
        package: ContractPackageHash,
        nft_contract: String,
        token_id: String
    },
    OfferCreated {
        package: ContractPackageHash,
        buyer: Key,
        nft_contract: String,
        token_id: String,
        price: U256
    },
    OfferWithdraw {
        package: ContractPackageHash,
        buyer: Key,
        nft_contract: String,
        token_id: String
    },
    OfferAccepted {
        package: ContractPackageHash,
        seller: Key,
        buyer: Key,
        nft_contract: String,
        token_id: String,
        price: U256
    },
}