pub mod constants;
pub mod utils;

#[cfg(test)]
mod tests {
    use crate::constants::{
        TEST_BLOCK_TIME, PARAM_AMOUNT
    };
    use crate::utils::{
        arbitrary_user, arbitrary_user_key,  deploy_market, deploy_cep47,
        init_environment, deploy_erc20, execution_context, execution_error,
        fill_purse_on_token_contract, exec_deploy, 
         setup_context, simple_deploy_builder,
        test_public_key, mint_tokens, query,
    };
    use casper_execution_engine::core::{engine_state, execution};
    use casper_execution_engine::storage::global_state::StateProvider;
    use casper_types::{runtime_args, RuntimeArgs, U256};

    #[test]
    fn test_deploy_cep47() {
        let mut context = setup_context();
        deploy_cep47(&mut context.builder, context.account.address);
    }

    #[test]
    fn test_deploy_erc20() {
        let mut context = setup_context();
        deploy_erc20(&mut context.builder, context.account.address);
    }

    #[test]
    fn test_deploy_market() {
        let mut context = setup_context();
        deploy_market(&mut context.builder, context.account.address);
    }

    #[test]
    fn test_deploy_all() {
        let (context, _,_,cep47_hash,_,_,_) = init_environment();
        

        let data = query::<_, U256>(&context.builder, cep47_hash, "total_supply");
        println!("XXXX {:?}", data);
    }

    // #[test]
    // fn verify_bridge_entry_poitns() {
        // let mut context = setup_context();

        // let (bridge_address, _) = deploy_market(&mut context.builder, context.account.address);

        // let contract = context.builder.get_contract(bridge_address).unwrap();
        // let expected_entries = vec![
        //     EP_BRIDGE_IN,
        //     EP_BRIDGE_IN_CONFIRM,
        //     EP_CHECK_PARAMS,
        //     EP_BRIDGE_OUT,
        //     EP_TRANSFER_OUT,
        //     EP_WITHDRAW_COMMISSION,
        //     EP_SET_STABLE_COMMISSION_PERCENT,
        //     EP_GET_STABLE_COMMISSION_PERCENT,
        //     EP_SET_SIGNER,
        //     EP_GET_SIGNER,
        // ];

        // let mut count = 0;
        // for entry in contract.entry_points().keys() {
        //     assert!(expected_entries.contains(&entry.as_str()), "You have introduced a new entry point please add it to the expected list and cover with a tests");
        //     count += 1;
        // }
        // assert_eq!(count, expected_entries.len());
    // }

    #[test]
    fn set_stable_commission_percent_called_by_non_owner() {
        /*
            Scenario:
            1. Call "set_stable_commission_percent" entrypoint to set percent
            2. Assert fail
        */

        let mut context = setup_context();

        // Deploy the bridge contract
        let (market_hash, _) = deploy_market(&mut context.builder, context.account.address);

        // Try to set percent
        let user = arbitrary_user(&mut context);
        let deploy_item = simple_deploy_builder(user.address)
            .with_stored_session_hash(
                market_hash,
                "EP_SET_STABLE_COMMISSION_PERCENT",
                runtime_args! {
                    "PARAM_STABLE_COMMISSION_PERCENT" => "TEST_STABLE_COMMISSION_PERCENT()",
                },
            )
            .build();

        let error = execution_error(&mut context, deploy_item);

        let expected_error = engine_state::Error::Exec(execution::Error::InvalidContext);
        assert_eq!(error.to_string(), expected_error.to_string());
    }


}
