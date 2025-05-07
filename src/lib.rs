#[cfg(all(not(feature = "cw-v1"), not(feature = "cw-v2")))]
compile_error!("One of `cw-v1` or `cw-v2` must be enabled");

#[cfg(all(feature = "cw-v1", feature = "cw-v2"))]
compile_error!("Features `cw-v1` and `cw-v2` are mutually exclusive");

#[cfg(feature = "cw-v1")]
use cosmwasm_std_v1 as cosmwasm_std;
#[cfg(feature = "cw-v2")]
use cosmwasm_std_v2 as cosmwasm_std;

#[cfg(all(feature = "auth-v1", feature = "cw-v1"))]
use cw_storage_plus_v1 as cw_storage_plus;
#[cfg(all(feature = "auth-v2", feature = "cw-v2"))]
use cw_storage_plus_v2 as cw_storage_plus;

#[cfg(all(feature = "assets-v1", feature = "cw-v1"))]
use cw20_v1 as cw20;
#[cfg(all(feature = "assets-v2", feature = "cw-v2"))]
use cw20_v2 as cw20;

#[cfg(any(
    feature = "encryption-v1",
    feature = "encryption-v2",
    feature = "hashing-v1",
    feature = "hashing-v2"
))]
mod private_communication;

#[cfg(feature = "any")]
pub mod any;
#[cfg(any(feature = "assets-v1", feature = "assets-v2"))]
pub mod assets;
#[cfg(any(feature = "auth-v1", feature = "auth-v2"))]
pub mod auth;
#[cfg(feature = "bech32")]
pub mod bech32;
#[cfg(any(feature = "encryption-v1", feature = "encryption-v2"))]
pub mod encryption;
#[cfg(any(feature = "hashing-v1", feature = "hashing-v2"))]
pub mod hashing;
#[cfg(feature = "nft")]
pub mod nft;

pub mod utils;
