pub mod constants;
pub mod utils;
use std::collections::BTreeMap;


#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::collections::{BTreeMap};
    use std::iter::Map;

    use crate::constants::{
        TEST_BLOCK_TIME, PARAM_AMOUNT, EP_CREATE_LISTING
    };
    use crate::utils::{
        arbitrary_user, arbitrary_user_key,  deploy_market, deploy_cep47,
        init_environment, deploy_erc20, execution_context, execution_error,
        fill_purse_on_token_contract, exec_deploy, 
         setup_context, simple_deploy_builder,
        test_public_key, mint_tokens, query, create_listing, approve_token, owner_of, query_dictionary,
    };
    use casper_execution_engine::core::{engine_state, execution};
    use casper_execution_engine::storage::global_state::StateProvider;
    use casper_types::bytesrepr::Bytes;
    use casper_types::{runtime_args, RuntimeArgs, U256, Key, CLTyped};

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
        

        let data: U256 = query(&context.builder, cep47_hash, "total_supply");
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
    fn create_listing_test() {
        /*
            Scenario:
            1. Call "create listing" entrypoint to create a listing
            2. Assert success
        */

        let min_bid_price = U256::one() * 3;
        let redemption_price = U256::one() * 10;
        let auction_duration: U256 = U256::one() * 86_400;
        let (mut context, _,_,cep47_hash,_,market_hash, market_package_hash) = init_environment();
        let approve_deploy = approve_token(cep47_hash, market_package_hash,  context.account.address);
        exec_deploy(&mut context, approve_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash, 
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration
        );
        exec_deploy(&mut context, create_listing_deploy).expect_success();

        // let data: Option<Key> = query_dictionary(&mut context.builder, cep47_hash, U256::one(), "owners");
        // println!("XXXX {:?}", data.unwrap());
        // println!("XXXX2 {:?}", market_hash);

        let res: BTreeMap<String, String> = query(&mut context.builder, market_hash, "latest_event");
                println!("resresres {:?}", res);

        assert_eq!(res.get("event_type").unwrap(), "market_listing_created");
        assert_eq!(res.get("contract_package_hash").unwrap(), &market_package_hash.to_string());
        assert_eq!(res.get("min_bid_price").unwrap(), &min_bid_price.to_string());
        assert_eq!(res.get("redemption_price").unwrap(), &redemption_price.to_string());
        assert_eq!(res.get("auction_duration").unwrap(), &auction_duration.to_string());
        assert_eq!(res.get("token_contract").unwrap(), &cep47_hash.to_formatted_string());

    }



    // NEGATIVE CASES:

    // #[test]
    // fn create_listing_test_invalid_auction_time() {
    //     /*
    //         Scenario:
    //         1. Call "create listing" entrypoint to create a listing
    //         2. Assert success
    //     */

    //     let min_bid_price: U256 = U256::one() * 3;
    //     let redemption_price: U256 = U256::one() * 10;
    //     let auction_duration: U256 = U256::zero();
    //     let (mut context, _,_,cep47_hash,_,market_hash, market_package_hash) = init_environment();
    //     let approve_deploy = approve_token(cep47_hash, market_package_hash,  context.account.address);
    //     exec_deploy(&mut context, approve_deploy).expect_success();

    //     let create_listing_deploy = create_listing(
    //         market_hash, 
    //         cep47_hash,
    //         context.account.address,
    //         min_bid_price, 
    //         redemption_price,
    //         auction_duration
    //     );


    // //    exec_deploy(&mut context, create_listing_deploy).expect_success();

    // //    let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
    // //        ERC20_INSUFFIENT_BALANCE_ERROR_CODE,
    // //    )));



    // }


}
