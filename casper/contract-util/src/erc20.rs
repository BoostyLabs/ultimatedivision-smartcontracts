use casper_contract::{
    contract_api::runtime::{call_versioned_contract, call_contract, self}, unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ContractPackageHash, Key, RuntimeArgs, U256, ContractHash};
use alloc::string::String;

mod consts {
    #![allow(dead_code)]

    pub(super) const EP_NAME: &str = "name";
    pub(super) const EP_SYMBOL: &str = "symbol";
    pub(super) const EP_DECIMALS: &str = "decimals";
    pub(super) const EP_BALANCE_OF: &str = "balance_of";
    pub(super) const EP_TRANSFER: &str = "transfer";
    pub(super) const EP_APPROVE: &str = "approve";
    pub(super) const EP_ALLOWANCE: &str = "allowance";
    pub(super) const EP_TRANSFER_FROM: &str = "transfer_from";
    pub(super) const EP_TOTAL_SUPPLY: &str = "total_supply";

    pub(super) const PARAM_ADDRESS: &str = "address";
    pub(super) const PARAM_OWNER: &str = "owner";
    pub(super) const PARAM_SPENDER: &str = "spender";
    pub(super) const PARAM_AMOUNT: &str = "amount";
    pub(super) const PARAM_RECIPIENT: &str = "recipient";
    pub(super) const PARAM_NAME: &str = "name";
    pub(super) const PARAM_SYMBOL: &str = "symbol";
    pub(super) const PARAM_DECIMALS: &str = "decimals";
    pub(super) const PARAM_TOTAL_SUPPLY: &str = "total_supply";
}

use consts::*;

pub fn transfer(contract: ContractPackageHash, recepient: Key, amount: U256) {
    let args = RuntimeArgs::try_new(|args| {
        args.insert(PARAM_RECIPIENT, recepient)?;
        args.insert(PARAM_AMOUNT, amount)?;
        Ok(())
    })
    .unwrap_or_revert();

    call_versioned_contract::<()>(contract, None, EP_TRANSFER, args);
}


pub fn transfer_from_contract_to_recipient(contract: ContractHash, recepient: Key, amount: U256) {

    let text = alloc::format!("recepient::: {:?}", recepient);
    runtime::print(&text);
    let args = RuntimeArgs::try_new(|args| {
        args.insert(PARAM_RECIPIENT, recepient)?;
        args.insert(PARAM_AMOUNT, amount)?;
        Ok(())
    })
    .unwrap_or_revert();

    call_contract::<()>(contract, EP_TRANSFER, args);
}


pub fn transfer_from(contract: ContractHash, owner: Key, recepient: Key, amount: U256) {

    let args = RuntimeArgs::try_new(|args| {
        args.insert(PARAM_OWNER, owner)?;
        args.insert(PARAM_RECIPIENT, recepient)?;
        args.insert(PARAM_AMOUNT, amount)?;
        Ok(())
    })
    .unwrap_or_revert();

    call_contract::<()>(contract, EP_TRANSFER_FROM, args);
}

pub fn transfer_from_by_package(contract: ContractPackageHash, owner: Key, recepient: Key, amount: U256) {

    let args = RuntimeArgs::try_new(|args| {
        args.insert(PARAM_OWNER, owner)?;
        args.insert(PARAM_RECIPIENT, recepient)?;
        args.insert(PARAM_AMOUNT, amount)?;
        Ok(())
    })
    .unwrap_or_revert();

    call_versioned_contract::<()>(contract, None, EP_TRANSFER_FROM, args);
}

pub fn balance_of(contract: ContractPackageHash, address: Key) -> U256 {

    let args = RuntimeArgs::try_new(|args| {
        args.insert(PARAM_ADDRESS, address)?;
        Ok(())
    })
    .unwrap_or_revert();
    call_versioned_contract::<U256>(contract, None, EP_BALANCE_OF, args)
}


pub fn name(contract: ContractPackageHash, address: Key) -> String {
    
    let args = RuntimeArgs::try_new(|args| {
        Ok(())
    })
    .unwrap_or_revert();
    call_versioned_contract::<String>(contract, None, EP_NAME, RuntimeArgs::new())
}

pub fn symbol(contract: ContractPackageHash, address: Key) -> String {
    
    let args = RuntimeArgs::try_new(|args| {
        Ok(())
    })
    .unwrap_or_revert();
    call_versioned_contract::<String>(contract, None, EP_SYMBOL, RuntimeArgs::new())
}

pub fn total_supply(contract: ContractPackageHash, address: Key) -> U256 {
    
    let args = RuntimeArgs::try_new(|args| {
        Ok(())
    })
    .unwrap_or_revert();
    call_versioned_contract::<U256>(contract, None, EP_TOTAL_SUPPLY, RuntimeArgs::new())
}
