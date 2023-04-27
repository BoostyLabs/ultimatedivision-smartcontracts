pub mod constants;
pub mod utils;


#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap};

    use crate::utils::{
        arbitrary_user, deploy_market, deploy_cep47,
        init_environment, deploy_erc20, execution_error,
        fill_purse_on_token_contract, exec_deploy, 
         setup_context, query, create_listing, approve_token, buy_listing, get_auction_data, query_balance,
    };
    use casper_execution_engine::core::{engine_state, execution};
    use casper_types::{U256, Key, ApiError};

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

        let (min_bid_price, redemption_price, auction_duration) = get_auction_data();
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

        let res: BTreeMap<String, String> = query(&mut context.builder, market_hash, "latest_event");

        println!("VVV::create_listing_test::res {:?}", res);

        assert_eq!(res.get("event_type").unwrap(), "market_listing_created");
        assert_eq!(res.get("contract_package_hash").unwrap(), &market_package_hash.to_string());
        assert_eq!(res.get("min_bid_price").unwrap(), &min_bid_price.to_string());
        assert_eq!(res.get("redemption_price").unwrap(), &redemption_price.to_string());
        assert_eq!(res.get("auction_duration").unwrap(), &auction_duration.to_string());
        assert_eq!(res.get("nft_contract").unwrap(), &cep47_hash.to_formatted_string());
        // vvv: check seller
        // vvv: check balances

    }


    #[test]
    fn buy_listing_test() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "buy listing" entrypoint to create a listing
            3. Assert success
        */

        let (
            mut context,
            erc20_hash,
            erc20_package_hash,
             cep47_hash,
             _,
             market_hash,
             market_package_hash
        ) = init_environment();
        let (min_bid_price, redemption_price, auction_duration) = get_auction_data();
        let token_id = "1";

        let approve_deploy = approve_token(
            cep47_hash,
            market_package_hash,
             context.account.address
        );
        exec_deploy(&mut context, approve_deploy).expect_success();

        let buyer = arbitrary_user(&mut context);
        fill_purse_on_token_contract(
            &mut context,
            erc20_hash,
            U256::one() * 10000,
            Key::from(buyer.address),
        );

        let seller_balance_before = query_balance(&mut context.builder, erc20_hash, &Key::from(context.account.address));
        let buyer_balance_before = query_balance(&mut context.builder, erc20_hash, &Key::from(buyer.address));

        let create_listing_deploy = create_listing(
            market_hash,
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration
        );
        exec_deploy(&mut context, create_listing_deploy).expect_success();

        let buy_listing_deploy = buy_listing(
            market_hash, 
            cep47_hash,
            erc20_package_hash,
            buyer.address,
            token_id
        );
        exec_deploy(&mut context, buy_listing_deploy).expect_success();

        let seller_balance_after = query_balance(&mut context.builder, erc20_hash, &Key::from(context.account.address));
        let buyer_balance_after = query_balance(&mut context.builder, erc20_hash, &Key::from(buyer.address));
        
        let res: BTreeMap<String, String> = query(&mut context.builder, market_hash, "latest_event");

        assert_eq!(res.get("event_type").unwrap(), "market_listing_purchased");
        assert_eq!(res.get("contract_package_hash").unwrap(), &market_package_hash.to_string());
        assert_eq!(res.get("min_bid_price").unwrap(), &min_bid_price.to_string());
        assert_eq!(res.get("auction_duration").unwrap(), &auction_duration.to_string());
        assert_eq!(res.get("nft_contract").unwrap(), &cep47_hash.to_formatted_string());
        assert_eq!(res.get("redemption_price").unwrap(), &redemption_price.to_string());
        assert_eq!(res.get("buyer").unwrap().to_string(), Key::Account(buyer.address).to_string());
        assert_eq!(
            &seller_balance_after.checked_sub(seller_balance_before).unwrap().to_string(), 
            res.get("redemption_price").unwrap()
        );
        assert_eq!(
            &buyer_balance_before.checked_sub(buyer_balance_after).unwrap().to_string(), 
            res.get("redemption_price").unwrap()
        );
    }



    // NEGATIVE CASES:

    #[test]
    fn create_listing_test_invalid_auction_time() {
        /*
            Scenario:
            1. Call "create listing" entrypoint to create a listing
            2. Assert success
        */

        let min_bid_price: U256 = U256::one() * 3;
        let redemption_price: U256 = U256::one() * 10;
        let auction_duration: U256 = U256::zero();
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
        let error = execution_error(&mut context, create_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1009,
        )));

        assert_eq!(error.to_string(), expected_error.to_string());

    }

    #[test]
    fn create_listing_test_invalid_redemption_price() {
        /*
            Scenario:
            1. Call "create listing" entrypoint to create a listing
            2. Assert success
        */

        let min_bid_price: U256 = U256::one() * 3;
        let redemption_price: U256 = U256::one() * 3;
        let auction_duration: U256 = U256::zero() * 86_400;
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
        let error = execution_error(&mut context, create_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1008,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }


}
