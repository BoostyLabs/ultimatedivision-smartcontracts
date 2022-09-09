use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
    DEFAULT_PAYMENT,
};
use casper_erc20::constants::AMOUNT_RUNTIME_ARG_NAME;
use casper_execution_engine::{
    core::{
        engine_state::{
            run_genesis_request::RunGenesisRequest, Error as EngineError, GenesisAccount,
        },
        execution::Error as ExecError,
    },
    storage::global_state::{CommitProvider, StateProvider},
};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, runtime_args, ContractHash, ContractPackageHash, Key,
    Motes, PublicKey, RuntimeArgs, SecretKey, U256, U512,
};
use k256::ecdsa::{recoverable, signature::Signer, SigningKey};
use sha3::{digest::Update, Digest, Keccak256};

const CONTRACT_ERC20_BYTES: &[u8] = include_bytes!("udtoken.wasm");

pub const SEC_TEST_KEY: &str = "1b95daad3a140364cc1f9fa7467d90a6ecaea17250ad0dfee8301a53d9089691";
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
}

pub fn setup_context() -> (UserAccount, InMemoryWasmTestBuilder) {
    // Create keypair.
    let secret_key = SecretKey::secp256k1_from_bytes(&hex::decode(SEC_TEST_KEY).unwrap()).unwrap();
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

    (account_data, builder)
}

pub fn deploy_contract<S>(
    builder: &mut WasmTestBuilder<S>,
    account: AccountHash,
    wasm_bytes: &[u8],
    deploy_args: RuntimeArgs,
    contract_key: &str,
) -> (ContractHash, ContractPackageHash)
where
    S: StateProvider + CommitProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_item = DeployItemBuilder::new()
        .with_empty_payment_bytes(runtime_args! {
            AMOUNT_RUNTIME_ARG_NAME => *DEFAULT_PAYMENT
        })
        .with_session_bytes(wasm_bytes.into(), deploy_args)
        .with_authorization_keys(&[account])
        .with_address(account)
        .build();

    let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();
    builder.exec(execute_request).commit();

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
    S: StateProvider + CommitProvider,
    EngineError: From<S::Error>,
    <S as StateProvider>::Error: Into<ExecError>,
{
    let deploy_args = runtime_args! {
        "key" => generate_verifying_key(),
    };

    deploy_contract(
        builder,
        account,
        CONTRACT_ERC20_BYTES,
        deploy_args,
        "erc20_token_contract",
    )
}

pub fn generate_verifying_key() -> String {
    let secret_key = SigningKey::from_bytes(&hex::decode(SEC_TEST_KEY).unwrap()).unwrap();
    let key = secret_key.verify_key();
    hex::encode(key.to_bytes())
}

pub fn sign(
    contract_hash: ContractHash,
    account_hash: AccountHash,
    nonce: U256,
    amount: U256,
) -> String {
    let secret_key = SigningKey::from_bytes(&hex::decode(SEC_TEST_KEY).unwrap()).unwrap();

    let digest = Keccak256::new()
        .chain(account_hash.as_bytes())
        .chain(contract_hash)
        .chain(amount.into_bytes().unwrap())
        .chain(nonce.into_bytes().unwrap())
        .finalize();

    let signature: recoverable::Signature = secret_key.sign(&digest);
    hex::encode(signature)
}

pub fn simple_deploy_builder(account: AccountHash) -> DeployItemBuilder {
    DeployItemBuilder::new()
        .with_empty_payment_bytes(runtime_args! {
            AMOUNT_RUNTIME_ARG_NAME => *DEFAULT_PAYMENT
        })
        .with_authorization_keys(&[account])
        .with_address(account)
}
