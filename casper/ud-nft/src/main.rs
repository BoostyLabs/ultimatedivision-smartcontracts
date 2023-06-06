#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;

use core::fmt::Display;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};
use casper_contract::{
    contract_api::{
        runtime::{self, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, CLType, CLTyped, CLValue, ContractHash,
    ContractPackageHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Group, Key,
    Parameter, RuntimeArgs, URef, U256, bytesrepr::{ToBytes, FromBytes}, account::AccountHash,
};
use cep47::{
    contract_utils::{ContractContext, Dict, OnChainContractStorage},
    Meta, TokenId, CEP47, Error,
};
use verifier::Verifier;

#[derive(Default)]
struct NFTToken(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for NFTToken {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl CEP47<OnChainContractStorage> for NFTToken {}
impl Verifier for NFTToken {}

impl NFTToken {
    fn constructor(&mut self, name: String, symbol: String, meta: Meta) {
        CEP47::init(self, name, symbol, meta);
        UUIDMapping::init();
    }
}

struct UUIDMapping {
    uuid_to_token_id_dict: Dict,
    token_id_to_uuid: Dict,
    last_id: URef,
}

impl UUIDMapping {
    const UUID_TO_TOKEN_MAPPING_DICT: &str = "uuid_mapping_dict";
    const TOKEN_ID_TO_UUID_MAPPING_DICT: &str = "token_id_to_uuid";

    const LAST_ID: &str = "last_used_id";

    fn instance() -> Self {
        UUIDMapping {
            uuid_to_token_id_dict: Dict::instance(Self::UUID_TO_TOKEN_MAPPING_DICT),
            token_id_to_uuid: Dict::instance(Self::TOKEN_ID_TO_UUID_MAPPING_DICT),
            last_id: *runtime::get_key(Self::LAST_ID)
                .unwrap_or_revert()
                .as_uref()
                .unwrap_or_revert(),
        }
    }

    fn init() {
        Dict::init(Self::UUID_TO_TOKEN_MAPPING_DICT);
        Dict::init(Self::TOKEN_ID_TO_UUID_MAPPING_DICT);
        runtime::put_key(
            Self::LAST_ID,
            Key::URef(storage::new_uref(U256::zero()).into_read_add_write()),
        );
    }

    fn get_token_id(&self, uuid: &str) -> Option<TokenId> {
        self.uuid_to_token_id_dict.get(uuid)
    }

    fn get_uuid(&self, token_id: TokenId) -> Option<String> {
        self.token_id_to_uuid.get(&token_id.to_string())
    }

    fn get_available_token_id(&self) -> TokenId {
        storage::read(self.last_id)
            .unwrap_or_revert()
            .unwrap_or_revert()
    }

    fn put_new_token(&self, uuid: &str) -> TokenId {
        let current_value: U256 = self.get_available_token_id();

        self.uuid_to_token_id_dict.set(uuid, current_value);
        self.token_id_to_uuid.set(&current_value.to_string(), uuid);
        storage::add(self.last_id, U256::one());
        current_value
    }

    fn get_token_ids(&self, token_ids: Vec<String>) -> Vec<TokenId> {
        token_ids
            .into_iter()
            .flat_map(|elem| self.get_token_id(&elem))
            .collect()
    }
}

#[no_mangle]
fn constructor() {
    let name = runtime::get_named_arg::<String>("name");
    let symbol = runtime::get_named_arg::<String>("symbol");
    let meta = runtime::get_named_arg::<Meta>("meta");
    NFTToken::default().constructor(name, symbol, meta);
}

#[no_mangle]
fn name() {
    let ret = NFTToken::default().name();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn symbol() {
    let ret = NFTToken::default().symbol();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn meta() {
    let ret = NFTToken::default().meta();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn total_supply() {
    let ret = NFTToken::default().total_supply();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn balance_of() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let ret = NFTToken::default().balance_of(owner);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn get_token_by_index() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let index = runtime::get_named_arg::<U256>("index");
    let ret = NFTToken::default()
        .get_token_by_index(owner, index)
        .and_then(|token_id| UUIDMapping::instance().get_uuid(token_id));
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn owner_of() {


    let token_id = runtime::get_named_arg::<String>("token_id");

    let ret = UUIDMapping::instance()
        .get_token_id(&token_id)
        .and_then(|token_id| NFTToken::default().owner_of(token_id));
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn token_meta() {
    let token_id = runtime::get_named_arg::<String>("token_id");
    let ret = UUIDMapping::instance()
        .get_token_id(&token_id)
        .and_then(|token_id| NFTToken::default().token_meta(token_id));
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn update_token_meta() {
    let token_id = runtime::get_named_arg::<String>("token_id");
    let token_meta = runtime::get_named_arg::<Meta>("token_meta");
    UUIDMapping::instance()
        .get_token_id(&token_id)
        .and_then(|token_id| {
            NFTToken::default()
                .set_token_meta(token_id, token_meta)
                .ok()
        })
        .unwrap_or_revert();
}

#[no_mangle]
fn burn() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let token_ids = runtime::get_named_arg::<Vec<String>>("token_ids");
    let token_ids = UUIDMapping::instance().get_token_ids(token_ids);
    NFTToken::default()
        .burn(owner, token_ids)
        .unwrap_or_revert();
}

#[no_mangle]
fn transfer() {
    let recipient = runtime::get_named_arg::<Key>("recipient");
    let token_ids = runtime::get_named_arg::<Vec<String>>("token_ids");
    let token_ids = UUIDMapping::instance().get_token_ids(token_ids);

    NFTToken::default()
        .transfer(recipient, token_ids)
        .unwrap_or_revert();
}

#[no_mangle]
fn transfer_from() {
    let sender = runtime::get_named_arg::<Key>("sender");
    let recipient = runtime::get_named_arg::<Key>("recipient");
    let token_ids = runtime::get_named_arg::<Vec<String>>("token_ids");
    let token_ids = UUIDMapping::instance().get_token_ids(token_ids);

    NFTToken::default()
        .transfer_from(sender, recipient, token_ids)
        .unwrap_or_revert();
}

#[no_mangle]
fn approve() {
    let spender_str = runtime::get_named_arg::<String>("spender");

    let spender = Key::from(ContractPackageHash::from_formatted_str(
        &spender_str
    ).unwrap_or_else(
        |e| runtime::revert(Error::ApprovalParseError))
    );

    let token_id = runtime::get_named_arg::<String>("token_id");
    let token_ids = UUIDMapping::instance().get_token_ids(vec![token_id]);

    NFTToken::default()
        .approve(spender, token_ids)
        .unwrap_or_revert();
}

#[no_mangle]
fn get_approved() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let token_id = runtime::get_named_arg::<String>("token_id");
    let ret = UUIDMapping::instance()
        .get_token_id(&token_id)
        .and_then(|token_id| NFTToken::default().get_approved(owner, token_id));
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn claim() {
    let signature = runtime::get_named_arg::<String>("signature");
    let token_uuid = runtime::get_named_arg::<String>("token_id");
    let uuid_mapping = UUIDMapping::instance();

    if let Some(_) = uuid_mapping.get_token_id(&token_uuid) {
        revert(cep47::Error::TokenIdAlreadyExists);
    }

    let mut nft = NFTToken::default();

    let capacity = nft.total_supply();

    if capacity > 10000.into() {
        revert(cep47::Error::TokenIdReachedLimit)
    }

    let caller = {
        let data = nft.get_caller();
        if let Some(caller) = data.into_account() {
            caller.as_bytes().to_vec()
        } else {
            data.into_hash().unwrap().to_vec()
        }
    };
    let contract_hash = ContractHash::from(nft.self_hash().unwrap_or_revert().into_hash().unwrap());

    nft.verify_uuid(signature, &token_uuid, contract_hash, &caller)
        .unwrap_or_else(|| revert(cep47::Error::InvalidSignature));
    let token_id = uuid_mapping.get_available_token_id();

    nft.mint(nft.get_caller(), vec![token_id], vec![Meta::new()])
        .unwrap_or_revert();
    uuid_mapping.put_new_token(&token_uuid);
}


#[no_mangle]
fn mint_one() {
    let token_uuid = runtime::get_named_arg::<String>("token_id");
    // let recipient = runtime::get_named_arg::<Key>("recipient");

    let recipient_str = runtime::get_named_arg::<String>("recipient");
    let recipient = Key::from(AccountHash::from_formatted_str(&recipient_str).unwrap_or_else(
        |e| runtime::revert(Error::MintSingleTokenParseError)
        )
    );


    let uuid_mapping = UUIDMapping::instance();

    if let Some(_) = uuid_mapping.get_token_id(&token_uuid) {
        revert(cep47::Error::TokenIdAlreadyExists);
    }

    let mut nft = NFTToken::default();

    let capacity = nft.total_supply();

    if capacity > 10000.into() {
        revert(cep47::Error::TokenIdReachedLimit)
    }

    let token_id = uuid_mapping.get_available_token_id();

    nft.mint(recipient, vec![token_id], vec![Meta::new()])
        .unwrap_or_revert();

    uuid_mapping.put_new_token(&token_uuid);
}

#[no_mangle]
fn call() {
    // Read arguments for the constructor call.
    let verifying_key: String = runtime::get_named_arg("key");
    let mut default_meta = Meta::new();
    default_meta.insert("baseURI".to_owned(), "placeholder".to_owned());

    // Prepare constructor args
    let constructor_args = runtime_args! {
        "name" => "Ultimate Division NFT",
        "symbol" => "UDNFT",
        "meta" => default_meta,
    };

    let mut named_keys = NamedKeys::new();
    verifier::add_key(&mut named_keys, verifying_key);

    let (contract_hash, _) = storage::new_contract(
        get_entry_points(),
        Some(named_keys),
        Some(String::from("contract_package_hash")),
        None,
    );

    let package_hash: ContractPackageHash = ContractPackageHash::new(
        runtime::get_key("contract_package_hash")
            .unwrap_or_revert()
            .into_hash()
            .unwrap_or_revert(),
    );

    let constructor_access: URef =
        storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
            .unwrap_or_revert()
            .pop()
            .unwrap_or_revert();

    let _: () = runtime::call_contract(contract_hash, "constructor", constructor_args);

    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
        .unwrap_or_revert();

    runtime::put_key("ultima_division_nft_contract_hash", contract_hash.into());

    runtime::put_key(
        "ultima_division_nft_contract_hash_wrapped",
        storage::new_uref(contract_hash).into(),
    );
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("name", String::cl_type()),
            Parameter::new("symbol", String::cl_type()),
            Parameter::new("meta", Meta::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "name",
        vec![],
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "symbol",
        vec![],
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "meta",
        vec![],
        Meta::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "total_supply",
        vec![],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "balance_of",
        vec![Parameter::new("owner", Key::cl_type())],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "owner_of",
        vec![Parameter::new("token_id", String::cl_type())],
        CLType::Option(Box::new(CLType::Key)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "burn",
        vec![
            Parameter::new("owner", Key::cl_type()),
            Parameter::new("token_ids", CLType::List(Box::new(String::cl_type()))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "transfer",
        vec![
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new("token_ids", CLType::List(Box::new(String::cl_type()))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "transfer_from",
        vec![
            Parameter::new("sender", Key::cl_type()),
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new("token_ids", CLType::List(Box::new(String::cl_type()))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "approve",
        vec![
            Parameter::new("spender", String::cl_type()),
            Parameter::new("token_id", String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_approved",
        vec![
            Parameter::new("owner", Key::cl_type()),
            Parameter::new("token_id", String::cl_type()),
        ],
        CLType::Option(Box::new(CLType::Key)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_token_by_index",
        vec![
            Parameter::new("owner", Key::cl_type()),
            Parameter::new("index", U256::cl_type()),
        ],
        CLType::Option(Box::new(String::cl_type())),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "claim",
        vec![
            Parameter::new("signature", String::cl_type()),
            Parameter::new("token_id", String::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "mint_one",
        vec![
            Parameter::new("token_id", String::cl_type()),
            Parameter::new("recipient", String::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}
