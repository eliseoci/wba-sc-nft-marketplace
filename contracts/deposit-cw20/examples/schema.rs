use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use deposit_cw20::msg::{Cw20DepositResponse, Cw721DepositResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use deposit_cw20::state::{Cw20Deposits, Cw721Deposits};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Cw20Deposits), &out_dir);
    export_schema(&schema_for!(Cw721Deposits), &out_dir);
    export_schema(&schema_for!(Cw20DepositResponse), &out_dir);
    export_schema(&schema_for!(Cw721DepositResponse), &out_dir);
}
