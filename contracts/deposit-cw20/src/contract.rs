#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128, WasmMsg
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{Cw20DepositResponse, Cw721DepositResponse, ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg, Cw721HookMsg, BidsResponse};
use crate::state::{Cw20Deposits, CW20_DEPOSITS, Cw721Deposits, CW721_DEPOSITS, Offer, ASKS, Bid, BIDS};

const CONTRACT_NAME: &str = "deposit-cw20-example";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => receive_cw20(deps, _env, info, cw20_msg),
        ExecuteMsg::ReceiveNft(cw721_msg) => receive_cw721(deps, _env, info, cw721_msg),
        ExecuteMsg::WithdrawNft { contract, token_id } => execute_cw721_withdraw(deps, info, contract, token_id),
        ExecuteMsg::WithdrawBid { contract, token_id } => execute_withdraw_bid(deps, info, contract, token_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Cw20Deposits { address } => to_binary(&query_cw20_deposits(deps, address)?),
        QueryMsg::Cw721Deposits { address, contract } => to_binary(&query_cw721_deposits(deps, address, contract)?),
        QueryMsg::Bids { cw721_contract, token_id} => to_binary(&query_bids(deps, cw721_contract, token_id)?)
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {owner, amount }) => execute_cw20_deposit(deps, info, owner, amount),
        Ok(Cw20HookMsg::Purchase { cw721_contract, token_id }) => execute_purchase(deps, info, cw721_contract, token_id, cw20_msg),
        Ok(Cw20HookMsg::PlaceBid { cw721_contract, token_id }) => execute_place_bid(deps, info, cw721_contract, token_id, cw20_msg),
        _ => Err(ContractError::CustomError { val: "Invalid Cw20HookMsg".to_string() }),
    }
}

pub fn receive_cw721(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw721_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw721_msg.msg) {
        Ok(Cw721HookMsg::Deposit {owner, token_id, cw20_contract, amount }) => execute_cw721_deposit(deps, info, owner, token_id, cw20_contract, amount),
        _ => Err(ContractError::CustomError { val: "Invalid Cw721HookMsg".to_string() }),
    }
}

pub fn execute_purchase(deps: DepsMut, _info:MessageInfo, cw721_contract:String, token_id:String, msg:Cw20ReceiveMsg) -> Result<Response, ContractError> {
    match ASKS.load(deps.storage, (&cw721_contract, &token_id)) {
        Ok(ask) => {
            if msg.amount != Uint128::from(ask.amount) {
                return Err(ContractError::CustomError { val: "Invalid amount".to_string() });
            }

            let exe_msg = nft::contract::ExecuteMsg::TransferNft { recipient: msg.sender, token_id:token_id.clone() };
            let msg = WasmMsg::Execute { contract_addr: cw721_contract.clone(), msg: to_binary(&exe_msg)?, funds:vec![] };

            CW721_DEPOSITS.remove(deps.storage, (&cw721_contract, &ask.owner, &token_id));
            ASKS.remove(deps.storage, (&cw721_contract, &token_id));

            Ok(Response::new()
            .add_attribute("execute", "purchase")
            .add_message(msg))
        }
        Err(_) => {
            return Err(ContractError::CustomError { val: "No such ask".to_string() });
        }
    }
}

pub fn execute_place_bid(deps: DepsMut, info:MessageInfo, cw721_contract:String, token_id:String, msg:Cw20ReceiveMsg) -> Result<Response, ContractError> {
    let cw20_contract = info.sender.to_string();

    match ASKS.load(deps.storage, (&cw721_contract, &token_id)) {
        Ok(ask) => {
            if msg.amount >= Uint128::from(ask.amount) {
                return Err(ContractError::CustomError { val: "Bid is equal or higher than current asking price. Execute Purchase NFT instead.".to_string() });
            }
        }
        Err(_) => ()
    };

    match BIDS.load(deps.storage, (&cw20_contract, &token_id)) {
        Ok(bid) => {
            if msg.amount <= Uint128::from(bid.amount) {
                return Err(ContractError::CustomError { val: "Bid amount needs to be higher than current bid".to_string() });
            }
        }
        Err(_) => ()
    }
    
    let bid = Bid { 
        bidder: msg.sender,
        cw721_contract: cw721_contract.clone(),
        token_id: token_id.clone(),
        cw20_contract,
        amount: msg.amount.u128()
    };
    BIDS.save(deps.storage, (&cw721_contract, &token_id), &bid)?;

    Ok(Response::new()
    .add_attribute("execute", "place_bid"))
}

pub fn execute_withdraw_bid(
    deps: DepsMut,
    info: MessageInfo,
    contract:String,
    token_id: String,
) -> Result<Response, ContractError> {
    let bidder = info.sender.clone().into_string();
    let bid = match BIDS.load(deps.storage, (&contract, &token_id)) {
        Ok(bid) => {
            if bidder != bid.bidder {
                return Err(ContractError::CustomError { val: "Unauthorized".to_string() });
            }
            bid
        }
        Err(_) => return Err(ContractError::NoBidToWithdraw {  })
    };
    let exe_msg = cw20_base::msg::ExecuteMsg::Transfer { recipient: bidder, amount: Uint128::from(bid.amount) };
    let msg = WasmMsg::Execute { contract_addr: bid.cw20_contract.clone(), msg: to_binary(&exe_msg)?, funds:vec![] };
    
    BIDS.remove(deps.storage, (&contract, &token_id));
    
    Ok(Response::new()
    .add_attribute("execute", "withdraw_bid")
    .add_message(msg))
}

pub fn execute_cw20_deposit(deps: DepsMut, info: MessageInfo, owner:String, amount:u128) -> Result<Response, ContractError> {
    let sender = info.sender.clone().into_string();
    //check to see if u
    match CW20_DEPOSITS.load(deps.storage, (&owner, &sender)) {
        Ok(mut deposit) => {
            //add coins to their account
            deposit.amount = deposit.amount.checked_add(amount).unwrap();
            deposit.count = deposit.count.checked_add(1).unwrap();
            CW20_DEPOSITS
                .save(deps.storage, (&owner, &sender), &deposit)
                .unwrap();
        }
        Err(_) => {
            //user does not exist, add them.
            let deposit = Cw20Deposits {
                count: 1,
                owner: owner.clone(),
                contract:info.sender.into_string(),
                amount
            };
            CW20_DEPOSITS
                .save(deps.storage, (&owner, &sender), &deposit)
                .unwrap();
        }
    }
    Ok(Response::new()
        .add_attribute("execute", "cw20_deposit")
        .add_attribute("owner", owner)
        .add_attribute("contract", sender.to_string())
        .add_attribute("amount", amount.to_string()))
}

pub fn execute_cw20_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    contract:String,
    amount: u128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone().into_string();
    match CW20_DEPOSITS.load(deps.storage, (&sender, &contract)) {
        Ok(mut deposit) => {
            //add coins to their account
            deposit.amount = deposit.amount.checked_sub(amount).unwrap();
            deposit.count = deposit.count.checked_sub(1).unwrap();
            CW20_DEPOSITS
                .save(deps.storage, (&sender, &contract), &deposit)
                .unwrap();

            let exe_msg = cw20_base::msg::ExecuteMsg::Transfer { recipient: sender, amount: Uint128::from(amount) };
            let msg = WasmMsg::Execute { contract_addr: contract, msg: to_binary(&exe_msg)?, funds:vec![] };

            Ok(Response::new()
            .add_attribute("execute", "withdraw")
            .add_message(msg))
        }
        Err(_) => {
            return Err(ContractError::NoCw20ToWithdraw {  });
        }
    }
}

pub fn execute_cw721_deposit(deps: DepsMut, info: MessageInfo, owner:String, token_id:String, cw20_contract:String, amount:u128) -> Result<Response, ContractError> {
    let cw721_contract = info.sender.clone().into_string();
    //check to see if u

    if CW721_DEPOSITS.has(deps.storage, (&cw721_contract, &owner, &token_id)) == true {
        return Err(ContractError::CustomError { val: "Already deposited".to_string() });
    }

    let deposit = Cw721Deposits {
        owner: owner.clone(),
        contract:info.sender.into_string(),
        token_id:token_id.clone()
    };
    CW721_DEPOSITS
        .save(deps.storage, (&cw721_contract, &owner, &token_id), &deposit)
        .unwrap();
    
    let ask = Offer {
        owner: owner.clone(),
        amount,
        cw20_contract,
        cw721_contract:cw721_contract.clone(),
        token_id:token_id.clone()
    };

    ASKS.save(deps.storage, (&cw721_contract, &token_id), &ask).unwrap();

    Ok(Response::new()
        .add_attribute("execute", "cw721_deposit")
        .add_attribute("owner", owner)
        .add_attribute("contract", cw721_contract.to_string())
        .add_attribute("token_id", token_id.to_string()))
}

pub fn execute_cw721_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    contract:String,
    token_id: String,
) -> Result<Response, ContractError> {
    let owner = info.sender.clone().into_string();
    if CW721_DEPOSITS.has(deps.storage, (&contract, &owner, &token_id)) == false {
        return Err(ContractError::NoCw721ToWithdraw {  });
    }

    CW721_DEPOSITS.remove(deps.storage, (&contract, &owner, &token_id));
    ASKS.remove(deps.storage, (&contract, &token_id));
    let exe_msg = nft::contract::ExecuteMsg::TransferNft { recipient: owner, token_id: token_id };
    let msg = WasmMsg::Execute { contract_addr: contract, msg: to_binary(&exe_msg)?, funds:vec![] };

    Ok(Response::new()
    .add_attribute("execute", "withdraw")
    .add_message(msg))
}

fn query_cw20_deposits(deps: Deps, address: String) -> StdResult<Cw20DepositResponse> {
    let res: StdResult<Vec<_>> = CW20_DEPOSITS
        .prefix(&address)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let deposits = res?;
    Ok(Cw20DepositResponse { deposits })
}

fn query_cw721_deposits(deps: Deps, address: String, contract:String) -> StdResult<Cw721DepositResponse> {
    let res: StdResult<Vec<_>> = CW721_DEPOSITS
        .prefix((&contract, &address))
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let deposits = res?;
    Ok(Cw721DepositResponse { deposits })
}

fn query_bids(deps: Deps, cw721_contract: String, token_id: String) -> StdResult<BidsResponse> {
    let bids = BIDS.may_load(deps.storage, (&cw721_contract, &token_id))?;
    Ok(BidsResponse { bids })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, from_binary};

    const SENDER: &str = "sender_address";
    const AMOUNT: u128 = 100000;
    const DENOM: &str = "utest";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info(SENDER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        println!("{:?}", res);
        assert_eq!(0, res.messages.len());
    }

    fn deposit_coins(deps: DepsMut) {
 
    }

    fn withdraw_coins(deps: DepsMut) {}


    #[test]
    fn _0_instantiate() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
    }

    #[test]
    fn _1_deposit() {

    }

    //Add code to query the deposits and check if they were properly stored
    #[test]
    fn _2_query_deposit() {

    }

    #[test]
    fn _1_deposit_then_withdraw() {

    }
}
