use std::{
    iter::repeat,
    sync::atomic::{AtomicUsize, Ordering},
    collections::BTreeMap
};

use casper_contract::contract_api::runtime::{call_versioned_contract, print};
use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
    DEFAULT_PAYMENT,
};

use casper_execution_engine::{
    core::{
        engine_state::{
            run_genesis_request::RunGenesisRequest, DeployItem, Error as EngineError,
            GenesisAccount,
        },
        execution::Error as ExecError,
    },
    storage::global_state::{in_memory::InMemoryGlobalState, StateProvider},
};

use casper_types::{
    account::AccountHash,
    bytesrepr::{ToBytes, FromBytes},
    runtime_args, ContractHash, ContractPackageHash, Key, Motes, PublicKey, RuntimeArgs, SecretKey,
    U256, CLTyped, U512, system::{handle_payment::ARG_TARGET, mint::ARG_ID},
};
// use contract_util::erc20;

// use contract_util::event::ContractEvent;

const CONTRACT_CEP47_BYTES: &[u8] = include_bytes!("../wasm/ud-nft.wasm");
const CONTRACT_ERC20_BYTES: &[u8] = include_bytes!("../wasm/erc20.wasm");
const CONTRACT_MARKET_BYTES: &[u8] = include_bytes!("../wasm/contract-market.wasm");

static DEPLOY_COUNTER: AtomicUsize = AtomicUsize::new(0);

use crate::constants::{
    TEST_ACCOUNT_BALANCE, TEST_BLOCK_TIME, TEST_ACCOUNT, PARAM_AMOUNT, PARAM_RECIPIENT, PARAM_MARKET_CONTRACT_NAME, EP_MINT, EP_CREATE_LISTING, EP_APPROVE, EP_BUY_LISTING, EP_MAKE_OFFER, EP_ACCEPT_OFFER, EP_FINAL_LISTING, EP_SET_COMMISSION_WALLET, PARAM_COMMISSION_WALLET, PARAM_STABLE_COMMISSION_PERCENT, EP_SET_STABLE_COMMISSION_PERCENT, TEST_COMMISSION_PERCENT
};
pub type Meta = BTreeMap<String, String>;
pub type TokenId = U256;

pub fn test_public_key() -> &'static str {
    include_str!("config/public_key.in")
}

pub fn test_signer_secret_key() -> &'static str {
    include_str!("config/signer_secret_key.in")
}

pub fn test_meta_nft() -> Meta {
    let mut meta = BTreeMap::new();
    meta.insert("color".to_string(), "red".to_string());
    meta
}

pub fn new_deploy_hash() -> [u8; 32] {
    let counter = DEPLOY_COUNTER.fetch_add(1, Ordering::SeqCst);
    let hash = repeat(counter)
        .take(4)
        .flat_map(|i| i.to_le_bytes())
        .collect::<Vec<_>>();
    hash.try_into().unwrap()
}

pub fn deploy_builder() -> DeployItemBuilder {
    DeployItemBuilder::new().with_deploy_hash(new_deploy_hash())
}

pub struct TestContext {
    pub account: UserAccount,
    pub builder: InMemoryWasmTestBuilder,
}

pub struct UserAccount {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: AccountHash,
}

impl UserAccount {
    fn new(secret_key: SecretKey) -> Self {
        let public_key = PublicKey::from(&secret_key);
        let address = AccountHash::from(&public_key);
        Self {
            secret_key,
            public_key,
            address,
        }
    }

    pub fn unique_account(context: &mut TestContext, unique_id: u8) -> Self {
        if unique_id == 255 {
            panic!("Account with id 255 booked for genesis account");
        }
        // Create a key using unique_id
        let secret_key = SecretKey::ed25519_from_bytes([unique_id; 32]).unwrap();
        let account = UserAccount::new(secret_key);

        // We need to transfer some funds to the account so it become active
        let deploy = simple_deploy_builder(context.account.address)
            .with_transfer_args(runtime_args![
                ARG_AMOUNT => U512::one() * TEST_ACCOUNT_BALANCE,
                ARG_TARGET => account.public_key.clone(),
                ARG_ID => Some(u64::from(unique_id))
            ])
            .build();
        context
            .builder
            .exec(ExecuteRequestBuilder::from_deploy_item(deploy).build())
            .commit()
            .expect_success();
        account
    }

    pub fn key(&self) -> Key {
        Key::from(self.address)
    }
}

pub fn deploy_contract<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
    wasm_bytes: &[u8],
    deploy_args: RuntimeArgs,
    contract_key: &str,
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_item = deploy_builder()
        .with_empty_payment_bytes(runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT
        })
        .with_session_bytes(wasm_bytes.into(), deploy_args)
        .with_authorization_keys(&[account])
        .with_address(account)
        .build();

    let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

    builder.exec(execute_request).commit().expect_success();

    let stored_account = builder.query(None, Key::Account(account), &[]).unwrap();
    
    let contract_hash = stored_account
        .as_account()
        .unwrap()
        .named_keys()
        .get(contract_key)
        .unwrap()
        .into_hash()
        .unwrap();

    let contract_package_hash = builder
        .query(None, Key::Hash(contract_hash), &[])
        .unwrap()
        .as_contract()
        .unwrap()
        .contract_package_hash();

    (ContractHash::new(contract_hash), contract_package_hash)
}

pub fn deploy_erc20<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_args = runtime_args! {
        "name" => "test token".to_string(),
        "symbol" => "TTKN",
        "decimals" => 9u8,
        "total_supply" => U256::one() * 10_000_000,
    };

    deploy_contract(
        builder,
        account,
        CONTRACT_ERC20_BYTES,
        deploy_args,
        "erc20_token_contract",
    )
}


pub fn deploy_nft<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_args = runtime_args! {
        "key" => "0f988af84bc00098933c61790ee888bf99d518ba116704b548d0f7ff3c7457fe",
    };

    deploy_contract(
        builder,
        account,
        CONTRACT_CEP47_BYTES,
        deploy_args,
        &"ultima_division_nft_contract_hash",
    )
}

pub fn deploy_market<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
    erc20_hash: ContractHash,
    commission_wallet: Key
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_args = runtime_args! {
        "erc20_hash" => erc20_hash,
        "commission_wallet" => commission_wallet
    };

    deploy_contract(
        builder,
        account,
        CONTRACT_MARKET_BYTES,
        deploy_args,
        &[PARAM_MARKET_CONTRACT_NAME, "contract_hash"].join("_"),
    )
}

pub fn balance_of(contract: ContractPackageHash, address: Key) -> U256 {
    let args = RuntimeArgs::try_new(|args| {
        args.insert("address", address)?;
        Ok(())
    }).unwrap();

    call_versioned_contract::<U256>(contract, None, "balance_of", args)
}

pub fn get_commission_wallet(context: &mut TestContext, unique_id: u8) -> Key {
    let commission_wallet_account = arbitrary_user( context, unique_id);
    let commission_wallet = Key::from(commission_wallet_account.address);
    commission_wallet
}

pub fn init_environment() -> (
    TestContext,
    ContractHash,
    ContractPackageHash,
    ContractHash,
    ContractPackageHash,
    ContractHash,
    ContractPackageHash,
    Key,
) {
    let mut context = setup_context();

    let commission_wallet = get_commission_wallet(&mut context, 100);
    let (
        erc20_hash, 
        erc20_package_hash
    ) = deploy_erc20::<InMemoryGlobalState>(&mut context.builder, context.account.address);

    let (
        cep47_hash,
        cep47_package_hash
    ) = deploy_nft::<InMemoryGlobalState>(&mut context.builder, context.account.address);

    let (
        market_hash, 
        market_package_hash
    ) = deploy_market::<InMemoryGlobalState>(
        &mut context.builder,
        context.account.address,
        erc20_hash,
        commission_wallet
    );

    let mint_deploy = mint_tokens(
        cep47_hash, 
        context.account.address,
        "one"
    );
    exec_deploy(&mut context, mint_deploy).expect_success();

    (
        context,
        erc20_hash,
        erc20_package_hash,
        cep47_hash,
        cep47_package_hash,
        market_hash,
        market_package_hash,
        commission_wallet
    )
}

pub fn setup_context() -> TestContext {
    // Create keypair.
    let secret_key = SecretKey::ed25519_from_bytes(TEST_ACCOUNT).unwrap();
    let account_data = UserAccount::new(secret_key);

    // Create a GenesisAccount.
    let account = GenesisAccount::account(
        account_data.public_key.clone(),
        Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
        None,
    );

    let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
    genesis_config.ee_config_mut().push_account(account);

    let run_genesis_request = RunGenesisRequest::new(
        *DEFAULT_GENESIS_CONFIG_HASH,
        genesis_config.protocol_version(),
        genesis_config.take_ee_config(),
    );

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&run_genesis_request).commit();

    TestContext {
        account: account_data,
        builder,
    }
}


pub fn simple_deploy_builder(account: AccountHash) -> DeployItemBuilder {
    deploy_builder()
        .with_empty_payment_bytes(runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT
        })
        .with_authorization_keys(&[account])
        .with_address(account)
}

pub fn dictionary_key<T: ToBytes>(value: &T) -> String {
    base64::encode(value.to_bytes().expect("infallible"))
}

pub fn query_dictionary<S, T, R>(
    builder: &mut WasmTestBuilder<S>,
    contract: ContractHash,
    value: T,
    key: &str
) -> R
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
    T: ToString,
    R: CLTyped + FromBytes + Sized
{
    let contract = builder
        .query(None, Key::Hash(contract.value()), &[])
        .unwrap()
        .as_contract()
        .cloned()
        .unwrap();

    let uref = contract
        .named_keys()
        .get(key)
        .unwrap()
        .as_uref()
        .cloned()
        .unwrap();

    let value = builder
        .query_dictionary_item(None, uref, &value.to_string())
        .unwrap()
        .as_cl_value()
        .cloned()
        .unwrap()
        .into_t::<R>()
        .unwrap();

    value
}

pub fn query<S, T>(
    builder: &WasmTestBuilder<S>,
    contract: ContractHash,
    key: &str,
) -> T
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
    T: CLTyped,
    T: FromBytes
{

    let value = builder
        .query(None, Key::Hash(contract.value()), &[String::from(key)])
        .unwrap()
        .as_cl_value()
        .cloned()
        .unwrap()
        .into_t::<T>()
        .unwrap();
    value
}


pub fn approve_nft(
    cep47_hash: ContractHash,
    spender: ContractPackageHash,
    account_address: AccountHash,
    token_id: &str
) -> DeployItem {

    println!("VVV1 {:?}", spender.to_formatted_string());
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            EP_APPROVE,
            runtime_args! {
                // "spender" => Key::from(spender),
                "spender" => spender.to_formatted_string(),
                "token_id" => token_id,
            },
        )
        .build()
        
}

pub fn approve_erc20(
    erc20_hash: ContractHash,
    spender: ContractPackageHash,
    account_address: AccountHash,
    amount: U256
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            erc20_hash,
            EP_APPROVE,
            runtime_args! {
                "spender" => Key::from(spender),
                "amount" => amount,
            },
        )
        .build()
        
}

pub fn mint_tokens(
    cep47_hash: ContractHash,
    account_address: AccountHash,
    token_id: &str
) -> DeployItem {


    println!("aaa {:?}", account_address.to_formatted_string());
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            EP_MINT,
            runtime_args! {
                "recipient" => account_address.to_formatted_string(),
                "token_id" => token_id,
            },
        )
        .build()
}

pub fn mint_erc20(
    cep47_hash: ContractHash,
    account_address: AccountHash
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            EP_MINT,
            runtime_args! {
                "recipient" => Key::from(account_address),
                "token_ids" => "one",
                "token_meta" => test_meta_nft(),
                "count" => 1
            },
        )
        .build()
}

pub fn create_listing(
    market_hash: ContractHash,
    cep47_hash: ContractHash,
    account_address: AccountHash,
    min_bid_price: U256,
    redemption_price: U256,
    auction_duration: U256,
    token_id: &str
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_CREATE_LISTING,
            runtime_args! {
                "nft_contract_hash" => get_nft_contract_hash(cep47_hash),
                "token_id" => token_id,
                "min_bid_price" => min_bid_price,
                "redemption_price" => redemption_price,
                "auction_duration" => auction_duration,
            },
        )
        .build()
}

pub fn make_offer(
    market_hash: ContractHash,
    cep47_hash: ContractHash,
    account_address: AccountHash,
    offer_price: U256,
    token_id: &str
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_MAKE_OFFER,
            runtime_args! {
                "nft_contract_hash" => get_nft_contract_hash(cep47_hash),
                "token_id" => token_id,
                "offer_price" => offer_price,
            },
        )
        .build()
}

pub fn accept_offer(
    market_hash: ContractHash,
    cep47_hash: ContractHash,
    account_address: AccountHash,
    token_id: &str
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_ACCEPT_OFFER,
            runtime_args! {
                "nft_contract_hash" => get_nft_contract_hash(cep47_hash),
                "token_id" => token_id,
            },
        )
        .build()
}

pub fn final_listing(
    market_hash: ContractHash,
    cep47_hash: ContractHash,
    account_address: AccountHash,
    token_id: &str
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_FINAL_LISTING,
            runtime_args! {
                "nft_contract_hash" => get_nft_contract_hash(cep47_hash),
                "token_id" => token_id,
            },
        )
        .build()
}

pub fn buy_listing(
    market_hash: ContractHash,
    cep47_hash: ContractHash,
    account_address: AccountHash,
    token_id: &str
) -> DeployItem {
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_BUY_LISTING,
            runtime_args! {
                "nft_contract_hash" => get_nft_contract_hash(cep47_hash),
                "token_id" => token_id
            },
        )
        .build()
}


pub fn set_commission_wallet(
    market_hash: ContractHash,
    account_address: AccountHash,
    commission_wallet: Key,
) -> DeployItem {
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_SET_COMMISSION_WALLET,
            runtime_args! {
                PARAM_COMMISSION_WALLET => commission_wallet,
            },
        )
        .build()
}

pub fn set_stable_commission_percent(
    market_hash: ContractHash,
    account_address: AccountHash,
    commission_percent: U256,
) -> DeployItem {
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_SET_STABLE_COMMISSION_PERCENT,
            runtime_args! {
                PARAM_STABLE_COMMISSION_PERCENT => commission_percent,
            },
        )
        .build()
}

pub fn get_auction_data() -> (U256, U256, U256) {
    let min_bid_price = U256::one() * 30;
    let redemption_price = U256::one() * 100;
    let auction_duration: U256 = U256::one() * 86_400;
    (min_bid_price, redemption_price, auction_duration)
}

pub fn get_commission(price: U256) -> U256 {
    price * TEST_COMMISSION_PERCENT / 100
}

pub fn get_price_minus_commission(redemption_price: U256) -> U256 {
    U256::one() * (redemption_price - (redemption_price * TEST_COMMISSION_PERCENT / 100))
}

pub fn owner_of(
    cep47_hash: ContractHash,
    account_address: AccountHash,
    token_id: &str
) -> DeployItem {

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            "owner_of",
            runtime_args! {
                "token_id" => token_id,
            },
        )
        .build()
}


//     value
// }
// pub fn read_contract_event<S, E>(
//     builder: &mut WasmTestBuilder<S>,
//     contract: ContractHash,
//     event_uref_name: &str,
// ) -> E
// where
//     S: StateProvider + CommitProvider,
//     EngineError: From<S::Error>,
//     <S as StateProvider>::Error: Into<ExecError>,
//     E: ContractEvent,
// {
//     let contract = builder
//         .query(None, Key::Hash(contract.value()), &[])
//         .unwrap()
//         .as_contract()
//         .cloned()
//         .unwrap();

//     let event_uref = contract
//         .named_keys()
//         .get(event_uref_name)
//         .unwrap()
//         .as_uref()
//         .cloned()
//         .unwrap();

//     let last_result = builder.last_exec_result();
//     let journal = match last_result {
//         ExecutionResult::Failure {
//             execution_journal, ..
//         } => execution_journal,
//         ExecutionResult::Success {
//             execution_journal, ..
//         } => execution_journal,
//     };

//     let mut event: Vec<E> = journal
//         .clone()
//         .into_iter()
//         .filter_map(|item| match item {
//             (Key::URef(uref), Transform::Write(StoredValue::CLValue(value)))
//                 if uref.addr() == event_uref.addr() =>
//             {
//                 let data: Bytes = value.into_t().unwrap();
//                 let (event, _) = E::from_bytes(&data).unwrap();
//                 Some(event)
//             }
//             _ => None,
//         })
//         .collect();
//     assert_eq!(event.len(), 1);
//     event.pop().unwrap()
// }

pub fn query_balance<S>(
    builder: &mut WasmTestBuilder<S>,
    contract: ContractHash,
    address: &Key,
) -> U256
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let contract = builder
        .query(None, Key::Hash(contract.value()), &[])
        .unwrap()
        .as_contract()
        .cloned()
        .unwrap();

    let balance_uref = contract
        .named_keys()
        .get("balances")
        .unwrap()
        .as_uref()
        .cloned()
        .unwrap();

    let balance = builder
        .query_dictionary_item(None, balance_uref, &dictionary_key(address))
        .unwrap()
        .as_cl_value()
        .cloned()
        .unwrap()
        .into_t::<U256>()
        .unwrap();

    balance
}

pub fn query_listing<S>(
    builder: &mut WasmTestBuilder<S>,
    contract: ContractHash,
    listing_id: &String,
) -> U256
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let contract = builder
        .query(None, Key::Hash(contract.value()), &[])
        .unwrap()
        .as_contract()
        .cloned()
        .unwrap();

    let balance_uref = contract
        .named_keys()
        .get("listings")
        .unwrap()
        .as_uref()
        .cloned()
        .unwrap();

    let balance = builder
        .query_dictionary_item(None, balance_uref, &dictionary_key(listing_id))
        .unwrap()
        .as_cl_value()
        .cloned()
        .unwrap()
        .into_t::<U256>()
        .unwrap();

    balance
}

pub fn fill_purse_on_token_contract(
    context: &mut TestContext,
    token_hash: ContractHash,
    amount: U256,
    recipient: Key,
) {
    // Transferings token on bridge token purse
    let deploy_item = simple_deploy_builder(context.account.address)
        .with_stored_session_hash(
            token_hash,
            "transfer",
            runtime_args! {
                PARAM_RECIPIENT => recipient,
                PARAM_AMOUNT => amount,
            },
        )
        .build();

    context
        .builder
        .exec(ExecuteRequestBuilder::from_deploy_item(deploy_item).build())
        .commit()
        .expect_success();

    // vvvfix: modify it    
    // let total_balance = query_balance(&mut context.builder, token_hash, &recipient);
    // assert_eq!(total_balance, initial_balance + amount);
}

pub fn arbitrary_user(context: &mut TestContext, unique_id: u8) -> UserAccount { // vvvref Default trait
    UserAccount::unique_account(context, unique_id)
}

pub fn arbitrary_user_key(context: &mut TestContext) -> Key {
    arbitrary_user(context, 0).key()
}

pub fn execution_context(
    context: &mut TestContext,
    deploy_item: DeployItem,
) -> &mut WasmTestBuilder<InMemoryGlobalState> {
    context
        .builder
        .exec(
            ExecuteRequestBuilder::from_deploy_item(deploy_item)
                .with_block_time(TEST_BLOCK_TIME)
                .build(),
        )
        .commit()
}

pub fn execution_error(context: &mut TestContext, deploy_item: DeployItem) -> EngineError {
    execution_context(context, deploy_item)
        .expect_failure()
        .get_error()
        .unwrap()
}

pub fn exec_deploy(
    context: &mut TestContext,
    deploy_item: DeployItem,
) -> &mut WasmTestBuilder<InMemoryGlobalState> {
    context
        .builder
        .exec(
            ExecuteRequestBuilder::from_deploy_item(deploy_item)
                .with_block_time(TEST_BLOCK_TIME) // tim: << return value of runtime::get_blocktime() is set here per-deploy
                .build(),
        )
        .commit()
}

pub fn get_context(
    context: &mut TestContext,
    deploy_item: DeployItem,
) -> &mut WasmTestBuilder<InMemoryGlobalState> {
    context
        .builder
        .exec(
            ExecuteRequestBuilder::from_deploy_item(deploy_item).build(),
        )
        .commit()
}

fn get_nft_contract_hash(hash: ContractHash) -> String {
    ["contract-", &hash.to_string()].join("")
}
