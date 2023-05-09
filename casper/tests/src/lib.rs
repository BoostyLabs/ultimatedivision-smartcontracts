pub mod constants;
pub mod utils;


#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap};

    use crate::utils::{
        arbitrary_user, deploy_market, deploy_cep47,
        init_environment, deploy_erc20, execution_error,
        fill_purse_on_token_contract, exec_deploy, 
         setup_context, query, create_listing, approve_nft, buy_listing, get_auction_data, query_balance, mint_tokens, make_offer, accept_offer, UserAccount, TestContext, approve_erc20
    };
    use casper_execution_engine::core::{engine_state, execution};
    use casper_types::{U256, Key, ApiError, ContractPackageHash, ContractHash};


    fn make_offer_flow(
        market_package_hash: ContractPackageHash, 
        market_hash: ContractHash,
        cep47_hash: ContractHash,
        erc20_hash: ContractHash,
        token_id: &str,
        offer_price: U256,
        approved_price: U256,
        context: &mut TestContext
    ) -> UserAccount {
        let (min_bid_price, redemption_price, auction_duration) = get_auction_data();
        // vvvfix: duplicate?    
        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash,
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        exec_deploy(context, create_listing_deploy).expect_success();


        let bidder = arbitrary_user(context);
        fill_purse_on_token_contract(
            context,
            erc20_hash,
            U256::one() * 10000,
            Key::from(bidder.address),
        );

        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(context, approve_nft_deploy).expect_success();

        let approve_erc20_deploy = approve_erc20(
            erc20_hash,
            market_package_hash,
            bidder.address,
            approved_price + 1 // vvvq
        );
        exec_deploy(context, approve_erc20_deploy).expect_success();

        let bidder_balance_before = query_balance(&mut context.builder, erc20_hash, &Key::from(bidder.address));

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            token_id
        );
        exec_deploy(context, make_offer_deploy).expect_success();

        let bidder_balance_after = query_balance(&mut context.builder, erc20_hash, &Key::from(bidder.address));
        println!("VVV::bidder_balance_before {:?}", bidder_balance_before);
        println!("VVV::bidder_balance_after {:?}", bidder_balance_after);


        assert_eq!(bidder_balance_before - offer_price, bidder_balance_after);



        bidder
    }


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
        let token_id = "1";

        let approve_nft_deploy = approve_nft(
            cep47_hash, market_package_hash,  context.account.address, vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash, 
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
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
        assert_eq!(res.get("seller").unwrap(), &Key::Account(context.account.address).to_string());
       // vvvrev: check nft balances

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

        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

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
            auction_duration,
            token_id
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
        // vvvrev: check NFT transfered
        // vvvrev: check listing query dictionary on market contract
        // vvvrev: check whether offers were cancelled
    }

    #[test]
    fn make_offer_test() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "make_offer" entrypoint to make an offer
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
        let offer_price = U256::one() * 4;
        let token_id = "1";



        let bidder = make_offer_flow(
            market_package_hash,
            market_hash,
            cep47_hash,
            erc20_hash,
            token_id,
            offer_price,
            offer_price,
            &mut context
        );


        let res: BTreeMap<String, String> = query(&mut context.builder, market_hash, "latest_event");

        println!("VVV-res {:?}", res);
        assert_eq!(res.get("event_type").unwrap(), "market_offer_created");
        assert_eq!(res.get("contract_package_hash").unwrap(), &market_package_hash.to_string());
        assert_eq!(res.get("buyer").unwrap().to_string(), Key::Account(bidder.address).to_string());
        assert_eq!(res.get("nft_contract").unwrap(), &cep47_hash.to_formatted_string());
        assert_eq!(res.get("token_id").unwrap(), &token_id);
        assert_eq!(res.get("redemption_price").unwrap(), &offer_price.to_string());
        // vvvrev: check collection of offers 

        // let make_offer_deploy: engine_state::DeployItem = make_offer(
        //     market_hash,
        //     cep47_hash,
        //     erc20_hash,
        //     bidder.address,
        //     offer_price + U256::one(), 
        //     token_id
        // );
        // exec_deploy(&mut context, make_offer_deploy).expect_success();
        // vvvrev: check collection of offers whether previous one was deleted



    }

    #[test]
    fn accept_offer_test() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "make_offer" entrypoint to make an offer
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
        let offer_price = U256::one() * 4;
        let token_id = "1";

        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash,
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        exec_deploy(&mut context, create_listing_deploy).expect_success();


        let bidder = arbitrary_user(&mut context);
        fill_purse_on_token_contract(
            &mut context,
            erc20_hash,
            U256::one() * 10000,
            Key::from(bidder.address),
        );

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            token_id
        );
        exec_deploy(&mut context, make_offer_deploy).expect_success();

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            token_id
        );
        exec_deploy(&mut context, make_offer_deploy).expect_success();

        println!("YYY- {:?}", bidder.address);
        let accept_offer_deploy: engine_state::DeployItem = accept_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            context.account.address,
            bidder.address, 
            token_id
        );
        exec_deploy(&mut context, accept_offer_deploy).expect_success();

        let res: BTreeMap<String, String> = query(&mut context.builder, market_hash, "latest_event");
        println!("VVV-res {:?}", res);
        assert_eq!(res.get("event_type").unwrap(), "market_offer_accepted");
        assert_eq!(res.get("contract_package_hash").unwrap(), &market_package_hash.to_string());
        assert_eq!(res.get("buyer").unwrap().to_string(), Key::Account(bidder.address).to_string());
        assert_eq!(res.get("seller").unwrap().to_string(), Key::Account(context.account.address).to_string());
        assert_eq!(res.get("nft_contract").unwrap(), &cep47_hash.to_formatted_string());
        assert_eq!(res.get("token_id").unwrap(), &token_id);
        assert_eq!(res.get("redemption_price").unwrap(), &offer_price.to_string());



    }

    // vvvcurrent: make_offer add negative cases
    // --------------------------------------------------- NEGATIVE CASES: ----------------------------------------------------  //

    #[test]
    fn create_listing_test_invalid_auction_time() {
        /*
            Scenario:
            1. Call "create listing" entrypoint to create a listing with invalid duration
            2. Assert fail
        */

        let min_bid_price: U256 = U256::one() * 3;
        let redemption_price: U256 = U256::one() * 10;
        let auction_duration: U256 = U256::one() * 100;
        let (mut context, _,_,cep47_hash,_,market_hash, market_package_hash) = init_environment();
        let token_id = "1";
        let approve_nft_deploy = approve_nft(
            cep47_hash, market_package_hash,  context.account.address, vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash, 
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
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
            1. Call "create listing" entrypoint to create a listing with invalid redemption price
            2. Assert fail
        */

        let min_bid_price: U256 = U256::one() * 3;
        let redemption_price: U256 = U256::one() * 3;
        let auction_duration: U256 = U256::zero() * 86_400;
        let (mut context, _,_,cep47_hash,_,market_hash, market_package_hash) = init_environment();
        let token_id = "1";
        let approve_nft_deploy = approve_nft(
            cep47_hash, market_package_hash,  context.account.address, vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash, 
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        let error = execution_error(&mut context, create_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1008,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }

    #[test]
    fn create_listing_test_not_approved_token() {
        /*
            Scenario:
            1. Call "create listing" entrypoint to create a listing without token approval
            2. Assert fail
        */

        let (min_bid_price, redemption_price, auction_duration) = get_auction_data();
        let (mut context, _,_,cep47_hash,_,market_hash, _) = init_environment();
        let token_id = "1";

        let create_listing_deploy = create_listing(
            market_hash, 
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        let error = execution_error(&mut context, create_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1007,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }

    #[test]
    fn create_listing_test_not_owner() {

        /*
            Scenario:
            1. Call "create listing" entrypoint to create a listing with token id which we don't own
            2. Assert fail
        */

        let (min_bid_price, redemption_price, auction_duration) = get_auction_data();
        let (mut context, _,_,cep47_hash,_,market_hash, market_package_hash) = init_environment();

        let invalid_token_id = "2";


        let user = arbitrary_user(&mut context);

        let mint_deploy = mint_tokens(
            cep47_hash, 
            user.address,
            vec![U256::one() * 2]
        );
        exec_deploy(&mut context, mint_deploy).expect_success();
    
        let approve_nft_deploy = approve_nft(
            cep47_hash, market_package_hash,  context.account.address, vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash, 
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            invalid_token_id
        );
        let error = execution_error(&mut context, create_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1003,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }

    #[test]
    fn buy_listing_test_invalid_id() {
        /*
            Scenario:
            1. Call "buy listing" entrypoint to create a listing with ionvalid listing id
            2. Assert fail
        */

        let (
            mut context,
            _,
            erc20_package_hash,
             cep47_hash,
             _,
             market_hash,
             _
        ) = init_environment();
        let invalid_token_id = "333";

        let buy_listing_deploy = buy_listing(
            market_hash, 
            cep47_hash,
            erc20_package_hash,
            context.account.address,
            invalid_token_id
        );
        let error = execution_error(&mut context, buy_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1000,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }

    #[test]
    fn buy_listing_test_invalid_price() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "buy listing" entrypoint with having insufficient balance
            3. Assert fail
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

        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let buyer = arbitrary_user(&mut context);
        fill_purse_on_token_contract(
            &mut context,
            erc20_hash,
            U256::one() * 3,
            Key::from(buyer.address),
        );

        let create_listing_deploy = create_listing(
            market_hash,
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        exec_deploy(&mut context, create_listing_deploy).expect_success();

        let buy_listing_deploy = buy_listing(
            market_hash, 
            cep47_hash,
            erc20_package_hash,
            buyer.address,
            token_id
        );

        let error = execution_error(&mut context, buy_listing_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1002,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }

    #[test]
    fn make_offer_invalid_listing() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "make_offer" entrypoint to make an offer
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
        let offer_price = U256::one() * 2;
        let token_id: &str = "1";
        let invalid_token_id = "222";

        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash,
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        exec_deploy(&mut context, create_listing_deploy).expect_success();


        let bidder = arbitrary_user(&mut context);
        fill_purse_on_token_contract(
            &mut context,
            erc20_hash,
            U256::one() * 10000,
            Key::from(bidder.address),
        );

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            invalid_token_id
        );
        let error = execution_error(&mut context, make_offer_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1000,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }
    
    #[test]
    fn make_offer_invalid_price() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "make_offer" entrypoint to make an offer
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
        let offer_price = U256::one() * 2;
        let token_id = "1";

        let approve_nft_deploy = approve_nft(
            cep47_hash,
            market_package_hash,
            context.account.address,
            vec![U256::one()]
        );
        exec_deploy(&mut context, approve_nft_deploy).expect_success();

        let create_listing_deploy = create_listing(
            market_hash,
            cep47_hash,
            context.account.address,
            min_bid_price, 
            redemption_price,
            auction_duration,
            token_id
        );
        exec_deploy(&mut context, create_listing_deploy).expect_success();


        let bidder = arbitrary_user(&mut context);
        fill_purse_on_token_contract(
            &mut context,
            erc20_hash,
            U256::one() * 10000,
            Key::from(bidder.address),
        );

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            token_id
        );
        let error = execution_error(&mut context, make_offer_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1011,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

    }
    // vvvrev: add make_offer use case to insufficient check balance

    #[test]
    fn make_offer_test_invalid_next_price() {
        /*
            Scenario:
            1. Call "create listing"
            2. Call "make_offer" entrypoint to make an offer
            3. Call "make_offer" entrypoint to make an offer with the price same as previous
            4. Assert fail
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
        let offer_price = U256::one() * 4;
        let token_id = "1";

        let bidder = make_offer_flow(
            market_package_hash, 
            market_hash,
            cep47_hash,
            erc20_hash,
            token_id,
            offer_price,
            offer_price,
            &mut context,
        );

        let bidder_balance_before = query_balance(&mut context.builder, erc20_hash, &Key::from(bidder.address));

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            token_id
        );
        let error = execution_error(&mut context, make_offer_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1012,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

        // vvvrev: check collection of offers whether previous one was deleted

        let bidder_balance_after = query_balance(&mut context.builder, erc20_hash, &Key::from(bidder.address));


        println!("VVV::bidder_balance_before {:?}", bidder_balance_before);
        println!("VVV::bidder_balance_after {:?}", bidder_balance_after);
    }

    #[test]
    fn make_offer_test_insufficient_balance() {
        // vvvinprogress - revision
        /*
            Scenario:
            1. Call "create listing"
            2. Call "make_offer" entrypoint to make an offer
            3. Call "make_offer" entrypoint to make an offer with the price same as previous
            4. Assert fail
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
        let offer_price = U256::one() * 50;
        let token_id = "1";

        let bidder = make_offer_flow(
            market_package_hash, 
            market_hash,
            cep47_hash,
            erc20_hash,
            token_id,
            offer_price,
            U256::one(),
            &mut context,
        );

        let bidder_balance_before = query_balance(&mut context.builder, erc20_hash, &Key::from(bidder.address));

        let make_offer_deploy: engine_state::DeployItem = make_offer(
            market_hash,
            cep47_hash,
            erc20_hash,
            bidder.address,
            offer_price, 
            token_id
        );
        let error = execution_error(&mut context, make_offer_deploy);
        // vvvrefactor: move codes
        let expected_error = engine_state::Error::Exec(execution::Error::Revert(ApiError::User(
            1012,
        )));
        assert_eq!(error.to_string(), expected_error.to_string());

        // vvvrev: check collection of offers whether previous one was deleted

        let bidder_balance_after = query_balance(&mut context.builder, erc20_hash, &Key::from(bidder.address));


        println!("VVV::bidder_balance_before {:?}", bidder_balance_before);
        println!("VVV::bidder_balance_after {:?}", bidder_balance_after);
    }

}
