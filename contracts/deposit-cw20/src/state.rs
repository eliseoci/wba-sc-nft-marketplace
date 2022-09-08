use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw20Deposits {
    pub count: i32,
    pub owner: String,
    pub contract:String,
    pub amount:u128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw721Deposits {
    pub owner: String,
    pub contract:String,
    pub token_id:String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Offer {
    pub owner:String,
    pub cw721_contract:String,
    pub token_id: String,
    pub cw20_contract:String,
    pub amount: u128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub bidder:String,
    pub cw721_contract:String,
    pub token_id: String,
    pub cw20_contract:String,
    pub amount: u128
}

//key is address, denom
pub const CW20_DEPOSITS: Map<(&str, &str), Cw20Deposits> = Map::new("cw20deposits");

//contract, owner, token_id
pub const CW721_DEPOSITS: Map<(&str, &str, &str), Cw721Deposits> = Map::new("cw721deposits");


//key can be cw721_contract, token_id
pub const BIDS: Map<(&str, &str), Bid> = Map::new("bids");
pub const ASKS: Map<(&str, &str), Offer> = Map::new("asks");
