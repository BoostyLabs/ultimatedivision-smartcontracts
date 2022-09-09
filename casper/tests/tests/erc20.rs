use casper_engine_test_support::ExecuteRequestBuilder;
use casper_erc20::constants::{
    CLAIM_ENTRY_POINT_NAME, CLAIM_VALUE_RUNTIME_ARG_NAME, NONCE_RUNTIME_ARG_NAME,
    SIGNATURE_RUNTIME_ARG_NAME,
};
use casper_types::{runtime_args, RuntimeArgs, U256};
use tests::{deploy_erc20, setup_context, simple_deploy_builder};

#[test]
fn deploy_test() {
    let (user, mut builder) = setup_context();
    deploy_erc20(&mut builder, user.address);
}

#[test]
fn claim() {
    let (user, mut builder) = setup_context();
    let (contract_hash, _) = deploy_erc20(&mut builder, user.address);
    let signature = tests::sign(contract_hash, user.address, U256::zero(), U256::one() * 10);

    let deploy = simple_deploy_builder(user.address)
        .with_stored_session_hash(
            contract_hash,
            CLAIM_ENTRY_POINT_NAME,
            runtime_args! {
                NONCE_RUNTIME_ARG_NAME => U256::zero(),
                CLAIM_VALUE_RUNTIME_ARG_NAME => U256::one() * 10,
                SIGNATURE_RUNTIME_ARG_NAME => signature
            },
        )
        .build();

    builder
        .exec(ExecuteRequestBuilder::from_deploy_item(deploy).build())
        .commit()
        .expect_success();
}

#[test]
fn claim_failure_because_nonce_is_invalid() {
    let (user, mut builder) = setup_context();
    let (contract_hash, _) = deploy_erc20(&mut builder, user.address);
    let signature = tests::sign(contract_hash, user.address, U256::one(), U256::one() * 10);

    let deploy = simple_deploy_builder(user.address)
        .with_stored_session_hash(
            contract_hash,
            CLAIM_ENTRY_POINT_NAME,
            runtime_args! {
                NONCE_RUNTIME_ARG_NAME => U256::one(), // Nonce expected to be zero
                CLAIM_VALUE_RUNTIME_ARG_NAME => U256::one() * 10,
                SIGNATURE_RUNTIME_ARG_NAME => signature
            },
        )
        .build();

    builder
        .exec(ExecuteRequestBuilder::from_deploy_item(deploy).build())
        .commit()
        .expect_failure();
}

#[test]
fn claim_failure_because_signature_is_invalid() {
    let (user, mut builder) = setup_context();
    let (contract_hash, _) = deploy_erc20(&mut builder, user.address);
    // Signature created for input 11 but contract called with 10
    let signature = tests::sign(contract_hash, user.address, U256::zero(), U256::one() * 11);

    let deploy = simple_deploy_builder(user.address)
        .with_stored_session_hash(
            contract_hash,
            CLAIM_ENTRY_POINT_NAME,
            runtime_args! {
                NONCE_RUNTIME_ARG_NAME => U256::zero(),
                CLAIM_VALUE_RUNTIME_ARG_NAME => U256::one() * 10,
                SIGNATURE_RUNTIME_ARG_NAME => signature
            },
        )
        .build();

    builder
        .exec(ExecuteRequestBuilder::from_deploy_item(deploy).build())
        .commit()
        .expect_failure();
}

#[test]
fn claim_fail_second_time_call_with_the_data() {
    let (user, mut builder) = setup_context();
    let (contract_hash, _) = deploy_erc20(&mut builder, user.address);
    let signature = tests::sign(contract_hash, user.address, U256::zero(), U256::one() * 10);

    let deploy = simple_deploy_builder(user.address)
        .with_stored_session_hash(
            contract_hash,
            CLAIM_ENTRY_POINT_NAME,
            runtime_args! {
                NONCE_RUNTIME_ARG_NAME => U256::zero(),
                CLAIM_VALUE_RUNTIME_ARG_NAME => U256::one() * 10,
                SIGNATURE_RUNTIME_ARG_NAME => signature.clone()
            },
        )
        .build();

    builder
        .exec(ExecuteRequestBuilder::from_deploy_item(deploy.clone()).build())
        .commit()
        .expect_success();

    builder
        .exec(ExecuteRequestBuilder::from_deploy_item(deploy).build())
        .commit()
        .expect_failure(); // This nonce is already used so it should fail

    let failing_deploy = simple_deploy_builder(user.address)
        .with_stored_session_hash(
            contract_hash,
            CLAIM_ENTRY_POINT_NAME,
            runtime_args! {
                NONCE_RUNTIME_ARG_NAME => U256::one(),
                CLAIM_VALUE_RUNTIME_ARG_NAME => U256::one() * 10,
                SIGNATURE_RUNTIME_ARG_NAME => signature
            },
        )
        .build();

    builder
        .exec(ExecuteRequestBuilder::from_deploy_item(failing_deploy).build())
        .commit()
        .expect_failure(); // We incremented nonce, but signature is for previous data so it should fail.
}
