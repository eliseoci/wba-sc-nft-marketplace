use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Invalid Owner")]
    InvalidOwner {},

    #[error("Invalid Coin")]
    InvalidCoin {},

    #[error("Bid from this address already exits for this token_id")]
    InvalidBid {},

    #[error("No bids from this sender for this token_id")]
    NoBidsForTokenID {},

    #[error("User does not have coins from this cw20 to withdraw")]
    NoCw20ToWithdraw {},

    #[error("Contract does not possess token_id from this cw721 to withdraw")]
    NoCw721ToWithdraw {},

    #[error("Contract does not possess bids for this token")]
    NoBidToWithdraw {},
}
