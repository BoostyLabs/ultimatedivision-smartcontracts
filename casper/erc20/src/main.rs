#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, CLAIM_VALUE_RUNTIME_ARG_NAME,
        NONCE_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SIGNATURE_RUNTIME_ARG_NAME, SPENDER_RUNTIME_ARG_NAME, TOKEN_CONTRACT_RUNTIME_ARG_NAME,
    },
    Address, ERC20,
};
use casper_types::{CLValue, Key, U256};

#[no_mangle]
pub extern "C" fn name() {
    let name = ERC20::default().name();
    runtime::ret(CLValue::from_t(name).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    let symbol = ERC20::default().symbol();
    runtime::ret(CLValue::from_t(symbol).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    let decimals = ERC20::default().decimals();
    runtime::ret(CLValue::from_t(decimals).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let total_supply = ERC20::default().total_supply();
    runtime::ret(CLValue::from_t(total_supply).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Address = runtime::get_named_arg(ADDRESS_RUNTIME_ARG_NAME);
    let balance = ERC20::default().balance_of(address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    ERC20::default()
        .transfer(recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn approve() {
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    ERC20::default().approve(spender, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let val = ERC20::default().allowance(owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    ERC20::default()
        .transfer_from(owner, recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn claim() {
    let amount: U256 = runtime::get_named_arg(CLAIM_VALUE_RUNTIME_ARG_NAME);
    let nonce: U256 = runtime::get_named_arg(NONCE_RUNTIME_ARG_NAME);
    let signature: String = runtime::get_named_arg(SIGNATURE_RUNTIME_ARG_NAME);
    ERC20::default()
        .claim(amount, nonce, signature)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn reclaim_token() {
    let token_contract: Key = runtime::get_named_arg(TOKEN_CONTRACT_RUNTIME_ARG_NAME);
    ERC20::default().reclaim_token(token_contract);
}

#[no_mangle]
pub extern "C" fn pause() {
    ERC20::default().pause();
}

#[no_mangle]
pub extern "C" fn unpause() {
    ERC20::default().unpause();
}

#[no_mangle]
pub extern "C" fn mint() {
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    let address: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    ERC20::default().mint(address, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn burn() {
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    let address: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    ERC20::default().burn(address, amount).unwrap_or_revert();
}

#[no_mangle]
fn call() {
    let key: String = runtime::get_named_arg("key");
    ERC20::install(
        "test".to_string(),
        "test".to_string(),
        10,
        U256::zero(),
        key,
    )
    .unwrap_or_revert();
}
