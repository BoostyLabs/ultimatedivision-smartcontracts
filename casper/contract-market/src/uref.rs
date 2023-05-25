//! Implementation of uref
use core::convert::TryInto;

use casper_contract::{contract_api::{storage, runtime}, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, URef, ApiError,
};

pub(crate) fn get_uref(name: &str) -> URef {
    let key = runtime::get_key(name)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();
    key.try_into().unwrap_or_revert()
}

fn uref(param_name: &str) -> URef {
    get_uref(param_name)
}

/// Reads a specific param
pub(crate) fn read<T: CLTyped + FromBytes>(param_name: &str) -> T {
    storage::read(uref(param_name))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

/// Writes a value to the specific param
pub(crate) fn write<T: CLTyped + ToBytes>(param_name: &str, value: T) {
    storage::write(uref(param_name), value);
}
