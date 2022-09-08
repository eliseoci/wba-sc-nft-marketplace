#[cfg(test)]
mod tests {
    use crate::helpers::DepositContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg, Cw20DepositResponse, Cw721HookMsg, Cw721DepositResponse, BidsResponse};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, to_binary};
    use cw20::{Cw20Contract, Cw20Coin, MinterResponse, BalanceResponse};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
    use cw20_base::msg::QueryMsg as Cw20QueryMsg;
    use cw721::OwnerOfResponse;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use cw20_example::{self};
    use nft::helpers::NftContract;
    use nft::{self};

    pub fn contract_deposit_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_example::contract::execute,
            cw20_example::contract::instantiate,
            cw20_example::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_nft() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            nft::contract::entry::execute,
            nft::contract::entry::instantiate,
            nft::contract::entry::query,
        );
        Box::new(contract)
    }

    const USER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const USER2: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaejxx23";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn store_code() -> (App, u64, u64, u64) {
        let mut app = mock_app();
        let deposit_id = app.store_code(contract_deposit_cw20());
        let cw20_id = app.store_code(contract_cw20());
        let cw721_id = app.store_code(contract_nft());
        (app, deposit_id, cw20_id, cw721_id)
    }

    fn deposit_instantiate(app: &mut App, deposit_id: u64) -> DepositContract {
        let msg = InstantiateMsg {};
        let deposit_contract_address = app
            .instantiate_contract(
                deposit_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "deposit-cw20",
                None,
            )
            .unwrap();
        DepositContract(deposit_contract_address)
    }

    fn cw_20_instantiate(app: &mut App, cw20_id:u64) -> Cw20Contract {
        let coin = Cw20Coin {address:USER.to_string(), amount:Uint128::from(10000u64)};
        let msg:Cw20InstantiateMsg = Cw20InstantiateMsg {decimals:10, name:"Token".to_string(), symbol:"TKN".to_string(), initial_balances:vec![coin], marketing:None, mint:None };
        let cw20_contract_address = app
        .instantiate_contract(
            cw20_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "cw20-example",
            None,
        )
        .unwrap();
    Cw20Contract(cw20_contract_address)
    }

    pub fn cw721_instantiate(app:&mut App, nft_id:u64, name:String, symbol:String, minter:String) -> NftContract {
        let contract = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(ADMIN),
                &nft::contract::InstantiateMsg { name, symbol, minter },
                &[],
                "nft",
                None,
            )
            .unwrap();
        NftContract(contract)
    }

    fn get_cw20_deposits(app: &App, deposit_contract: &DepositContract) -> Cw20DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Cw20Deposits { address: USER.to_string() })
            .unwrap()
    }

    fn get_bids(app: &App, deposit_contract: &DepositContract, cw721_contract: &NftContract, token_id: String) -> BidsResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Bids { cw721_contract: cw721_contract.addr().to_string(), token_id})
            .unwrap()
    }

    fn get_balance(app: &App, cw20_contract: &Cw20Contract, user:String) -> BalanceResponse {
        app.wrap()
            .query_wasm_smart(cw20_contract.addr(), &Cw20QueryMsg::Balance { address: user })
            .unwrap()
    }

    fn get_cw721_deposits(app: &App, deposit_contract: &DepositContract, nft_contract:&NftContract) -> Cw721DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Cw721Deposits { address: USER.to_string(), contract: nft_contract.addr().to_string() })
            .unwrap()
    }

    fn get_owner_of(app: &App, nft_contract:&NftContract, token_id:String) -> OwnerOfResponse {
        app.wrap()
            .query_wasm_smart(nft_contract.addr(), &nft::contract::QueryMsg::OwnerOf { token_id, include_expired: None })
            .unwrap()
    }

    fn mint_nft(app: &mut App, cw721_contract: &NftContract, token_id:String, token_uri:Option<String>, to:String) -> () {
        let mint_msg = nft::contract::MintMsg{token_id, owner:to, token_uri, extension:None };
        let msg = nft::contract::ExecuteMsg::Mint(mint_msg);
        let cosmos_msg = cw721_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    fn deposit_nft(app: &mut App, deposit_contract:&DepositContract, cw721_contract:&NftContract, cw20_contract: &Cw20Contract, token_id:String, amount:u128) {
        let hook_msg = Cw721HookMsg::Deposit { owner: USER.to_string(), token_id: "0".to_string(), cw20_contract: cw20_contract.addr().to_string(), amount };
        let msg = nft::contract::ExecuteMsg::SendNft { contract: deposit_contract.addr().to_string(), token_id: "0".to_string(), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw721_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    #[test]
    fn deposit_cw20() {
        let (mut app, deposit_id, cw20_id, _cw721_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        let balance = get_balance(&app, &cw20_contract, USER.to_string());
        println!("Intial Balance {:?}", balance);

        let hook_msg = Cw20HookMsg::Deposit { owner: USER.to_string(), amount: 500 };

        let msg = Cw20ExecuteMsg::Send { contract: deposit_contract.addr().to_string(), amount: Uint128::from(500u64), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw20_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

        let deposits = get_cw20_deposits(&app, &deposit_contract);
        println!("{:?}", deposits.deposits[0]);

        let balance = get_balance(&app, &cw20_contract, deposit_contract.addr().into_string());
        println!("Deposit Contract {:?}", balance);

        let balance = get_balance(&app, &cw20_contract, USER.to_string());
        println!("Post {:?}", balance);


    }

    #[test]
    fn mint_then_deposit_cw721_then_place_bid_then_withdraw_bid() {
        let (mut app, deposit_id, cw20_id, cw721_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);
        let cw721_contract = cw721_instantiate(&mut app, cw721_id, "NFT".to_string(), "NFT".to_string(), USER.to_string());
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        //mint a new NFT with token_id "0"
        mint_nft(&mut app, &cw721_contract, "0".to_string(), Some("url".to_string()), USER.to_string());


        //get owner of NFT with token_id "0"
        let owner = get_owner_of(&app, &cw721_contract, "0".to_string());
        println!("{:?}", owner);

        deposit_nft(&mut app, &deposit_contract, &cw721_contract, &cw20_contract, "0".to_string(), 500);

        //get owner of NFT with token_id "0"
        let owner = get_owner_of(&app, &cw721_contract, "0".to_string());
        println!("{:?}", owner);

        //get owner from deposits contract
        let deposits = get_cw721_deposits(&app, &deposit_contract, &cw721_contract);
        println!("{:?}", deposits);

        // gets user balance
        let balance = get_balance(&app, &cw20_contract, USER.to_string());
        println!("Intial Balance {:?}", balance);

        // place bid
        let hook_msg = Cw20HookMsg::PlaceBid { cw721_contract : cw721_contract.addr().to_string(), token_id: "0".to_string() };
        let msg = Cw20ExecuteMsg::Send { contract: deposit_contract.addr().to_string(), amount: Uint128::from(200u64), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw20_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();        

        let bids = get_bids(&app, &deposit_contract, &cw721_contract, "0".to_string());
        println!("Bids {:?}", bids);

        let balance = get_balance(&app, &cw20_contract, deposit_contract.addr().into_string());
        println!("Deposit Contract {:?}", balance);

        let balance = get_balance(&app, &cw20_contract, USER.to_string());
        println!("Post {:?}", balance);

        // withdraw bid
        let msg = ExecuteMsg::WithdrawBid { contract: cw721_contract.addr().to_string(), token_id: "0".to_string() };
        let cosmos_msg = deposit_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

        let bids = get_bids(&app, &deposit_contract, &cw721_contract, "0".to_string());
        println!("Bids {:?}", bids);
    }


}
