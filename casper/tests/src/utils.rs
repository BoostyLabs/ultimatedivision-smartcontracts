use std::{
    iter::repeat,
    sync::atomic::{AtomicUsize, Ordering},
    collections::BTreeMap
};

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
    U256, U512, CLTyped,
};

// use contract_util::event::ContractEvent;

const CONTRACT_CEP47_BYTES: &[u8] = include_bytes!("../wasm/cep47-token.wasm");
const CONTRACT_ERC20_BYTES: &[u8] = include_bytes!("../wasm/contract_erc20.wasm");
const CONTRACT_MARKET_BYTES: &[u8] = include_bytes!("../wasm/contract-market.wasm");

static DEPLOY_COUNTER: AtomicUsize = AtomicUsize::new(0);

use crate::constants::{
    TEST_ACCOUNT_BALANCE, TEST_BLOCK_TIME, TEST_ACCOUNT, PARAM_AMOUNT, PARAM_RECIPIENT, PARAM_NFT_NAME, PARAM_NFT_SYMBOL, PARAM_MARKET_CONTRACT_NAME, PARAM_NFT_CONTRACT_NAME, PARAM_NFT_PRICE, EP_MINT, EP_CREATE_LISTING, EP_APPROVE
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
                "target" => account.public_key.clone(),
                "id" => Some(u64::from(unique_id))
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

    println!("contract_keycontract_keycontract_key {}", contract_key);    
    
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
        "total_supply" => U256::max_value(),
    };

    deploy_contract(
        builder,
        account,
        CONTRACT_ERC20_BYTES,
        deploy_args,
        "erc20_token_contract",
    )
}


pub fn deploy_cep47<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_args = runtime_args! {
        "contract_name" => PARAM_NFT_CONTRACT_NAME,
        "name" => PARAM_NFT_NAME,
        "symbol" => PARAM_NFT_SYMBOL,
        "meta" => test_meta_nft(),
        "price" => PARAM_NFT_PRICE
    };

    deploy_contract(
        builder,
        account,
        CONTRACT_CEP47_BYTES,
        deploy_args,
        &[PARAM_NFT_CONTRACT_NAME, "contract_hash"].join("_"),
    )
}

pub fn deploy_market<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_args = runtime_args! {};

    // let (nft_address, _) = deploy_cep47(builder, account);
    // println!("AAA nft_address {nft_address}");

    deploy_contract(
        builder,
        account,
        CONTRACT_MARKET_BYTES,
        deploy_args,
        &[PARAM_MARKET_CONTRACT_NAME, "contract_hash"].join("_"),
    )
}

pub fn init_environment() -> (
    TestContext,
    ContractHash,
    ContractPackageHash,
    ContractHash,
    ContractPackageHash,
    ContractHash,
    ContractPackageHash,
) {
    let mut context = setup_context();

    let (
        erc20_hash, 
        erc20_package_hash
    ) = deploy_erc20::<InMemoryGlobalState>(&mut context.builder, context.account.address);

    let (
        cep47_hash,
        cep47_package_hash
    ) = deploy_cep47::<InMemoryGlobalState>(&mut context.builder, context.account.address);

    let (
        market_hash, 
        market_package_hash
    ) = deploy_market::<InMemoryGlobalState>(&mut context.builder, context.account.address);

    let mint_deploy = mint_tokens(
        cep47_hash, 
        context.account.address
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
    R: CLTyped + FromBytes
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


pub fn approve_token(
    cep47_hash: ContractHash,
    spender: ContractPackageHash,
    account_address: AccountHash
) -> DeployItem {

    println!("approve_token {:?}", spender.into_bytes());

    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            EP_APPROVE,
            runtime_args! {
                "spender" => Key::from(spender),
                "token_ids" => vec![U256::one()],
            },
        )
        .build()
        
}

pub fn mint_tokens(
    cep47_hash: ContractHash,
    account_address: AccountHash
) -> DeployItem {

    println!("AAAaccount_address {:?}", Key::from(account_address).to_string());
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            EP_MINT,
            runtime_args! {
                "recipient" => Key::from(account_address),
                "token_ids" => vec![U256::one()],
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
    price: U256
) -> DeployItem {

    println!("AAAaccount_address {:?}", Key::from(account_address).to_string());
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            market_hash,
            EP_CREATE_LISTING,
            runtime_args! {
                "token_contract_hash" => ["contract-", &cep47_hash.to_string()].join(""),
                "token_id" => "1",
                "price" => price,
            },
        )
        .build()
}

pub fn owner_of(
    cep47_hash: ContractHash,
    account_address: AccountHash
) -> DeployItem {

    println!("AAAaccount_address {:?}", Key::from(account_address).to_string());
    simple_deploy_builder(account_address)
        .with_stored_session_hash(
            cep47_hash,
            "owner_of",
            runtime_args! {
                "token_id" => "1",
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

    let balance: U256 = query_dictionary(&mut context.builder, token_hash, recipient, &dictionary_key(&"balances"));
    assert_eq!(balance, amount);
}

pub fn arbitrary_user(context: &mut TestContext) -> UserAccount {
    UserAccount::unique_account(context, 0)
}

pub fn arbitrary_user_key(context: &mut TestContext) -> Key {
    arbitrary_user(context).key()
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