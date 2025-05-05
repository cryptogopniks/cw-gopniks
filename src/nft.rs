use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    wasm_execute, Addr, CosmosMsg, Deps, QuerierWrapper, StdError, StdResult, Timestamp,
};

use thiserror::Error;

use crate::utils::convert_err;

#[cw_serde]
enum ExecuteMsg {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
    },
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
}

#[cw_serde]
#[derive(QueryResponses)]
enum QueryMsg {
    /// Return operator that can access all of the owner's tokens.
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    /// Return approvals that a token has
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    /// Return approval of a given operator for all tokens of an owner, error if not set
    #[returns(OperatorResponse)]
    Operator {
        owner: String,
        operator: String,
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
struct ApprovalResponse {
    pub approval: Approval,
}

#[cw_serde]
struct ApprovalsResponse {
    pub approvals: Vec<Approval>,
}

#[cw_serde]
struct OperatorResponse {
    pub approval: Approval,
}

#[cw_serde]
struct OperatorsResponse {
    pub operators: Vec<Approval>,
}

#[cw_serde]
struct TokensResponse {
    /// Contains all token_ids in lexicographical ordering
    /// If there are more than `limit`, use `start_after` in future queries
    /// to achieve pagination.
    pub tokens: Vec<String>,
}

#[cw_serde]
struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

/// Expiration represents a point in time when some event happens.
/// It can compare with a BlockInfo and will return is_expired() == true
/// once the condition is hit (and for every block in the future)
#[cw_serde]
#[derive(Copy)]
pub enum Expiration {
    /// AtHeight will expire when `env.block.height` >= height
    AtHeight(u64),
    /// AtTime will expire when `env.block.time` >= time
    AtTime(Timestamp),
    /// Never will never expire. Used to express the empty variant
    Never {},
}

pub fn check_tokens_holder(
    deps: Deps,
    holder: &Addr,
    collection_address: impl ToString,
    token_id_list: &[impl ToString],
) -> StdResult<()> {
    const MAX_LIMIT: u32 = 100;
    const ITER_LIMIT: u32 = 50;

    let mut token_list: Vec<String> = vec![];
    let mut token_amount_sum: u32 = 0;
    let mut i: u32 = 0;
    let mut last_token: Option<String> = None;

    while (i == 0 || token_amount_sum == MAX_LIMIT) && i < ITER_LIMIT {
        i += 1;

        let TokensResponse { tokens } = deps.querier.query_wasm_smart(
            collection_address.to_string(),
            &QueryMsg::Tokens {
                owner: holder.to_string(),
                start_after: last_token,
                limit: Some(MAX_LIMIT),
            },
        )?;

        for token in tokens.clone() {
            token_list.push(token);
        }

        token_amount_sum = tokens.len() as u32;
        last_token = tokens.last().cloned();
    }

    let are_tokens_owned = token_id_list
        .iter()
        .all(|x| token_list.contains(&x.to_string()));

    if !are_tokens_owned {
        Err(NftError::NftIsNotFound)?;
    }

    Ok(())
}

pub fn get_cw721_approve_all_msgs(
    querier: QuerierWrapper,
    collection_list: &[impl ToString],
    owner: impl ToString,
    operator: impl ToString,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msg_list: Vec<CosmosMsg> = vec![];

    for collection in collection_list {
        let OperatorsResponse { operators } = querier.query_wasm_smart(
            collection.to_string(),
            &QueryMsg::AllOperators {
                owner: owner.to_string(),
                include_expired: None,
                start_after: None,
                limit: None,
            },
        )?;

        let target_operator = operators
            .iter()
            .find(|x| x.spender.to_string() == operator.to_string());

        if target_operator.is_none() {
            msg_list.push(
                wasm_execute(
                    collection.to_string(),
                    &ExecuteMsg::ApproveAll {
                        operator: operator.to_string(),
                        expires: None,
                    },
                    vec![],
                )
                .map(CosmosMsg::Wasm)?,
            );
        }
    }

    Ok(msg_list)
}

pub fn get_cw721_transfer_msg(
    collection: impl Into<String>,
    recipient: impl ToString,
    token_id: impl ToString,
) -> StdResult<CosmosMsg> {
    wasm_execute(
        collection,
        &ExecuteMsg::TransferNft {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        },
        vec![],
    )
    .map(CosmosMsg::Wasm)
}

pub fn get_cw721_mint_msg(
    collection: impl Into<String>,
    recipient: impl ToString,
    token_id: impl ToString,
) -> StdResult<CosmosMsg> {
    wasm_execute(
        collection,
        &ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: recipient.to_string(),
            token_uri: None,
        },
        vec![],
    )
    .map(CosmosMsg::Wasm)
}

pub fn get_cw721_burn_msg(
    collection: impl Into<String>,
    token_id: impl ToString,
) -> StdResult<CosmosMsg> {
    wasm_execute(
        collection,
        &ExecuteMsg::Burn {
            token_id: token_id.to_string(),
        },
        vec![],
    )
    .map(CosmosMsg::Wasm)
}

#[derive(Error, Debug, PartialEq)]
pub enum NftError {
    #[error("NFT isn't found!")]
    NftIsNotFound,

    #[error("Collection isn't found!")]
    CollectionIsNotFound,

    #[error("Empty token list!")]
    EmptyTokenList,

    #[error("Empty collection list!")]
    EmptyCollectionList,

    #[error("NFT already is added!")]
    NftDuplication,

    #[error("Collection already exists!")]
    CollectionDuplication,

    #[error("Incorrect token list!")]
    IncorrectTokenList,

    #[error("Incorrect token list!")]
    IncorrectCollectionList,

    #[error("Max token amount per tx is exceeded!")]
    ExceededTokenLimit,

    #[error("Collection isn't added!")]
    CollectionIsNotAdded,
}

impl From<NftError> for StdError {
    fn from(error: NftError) -> Self {
        convert_err(error)
    }
}
