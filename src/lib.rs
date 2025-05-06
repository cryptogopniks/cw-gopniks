#[cfg(all(not(feature = "cw-v1"), not(feature = "cw-v2")))]
compile_error!("One of `cw-v1` or `cw-v2` must be enabled");

#[cfg(all(feature = "cw-v1", feature = "cw-v2"))]
compile_error!("Features `cw-v1` and `cw-v2` are mutually exclusive");

#[cfg(feature = "cw-v1")]
use cosmwasm_std_v1 as cosmwasm_std;
#[cfg(feature = "cw-v2")]
use cosmwasm_std_v2 as cosmwasm_std;

#[cfg(feature = "cw-v1")]
use cw20_v1 as cw20;
#[cfg(feature = "cw-v2")]
use cw20_v2 as cw20;

mod utils;

#[cfg(any(feature = "encryption", feature = "hashing"))]
mod private_communication;

#[cfg(feature = "assets")]
pub mod assets;
// #[cfg(feature = "auth")]
// pub mod auth;
// #[cfg(feature = "bech32")]
// pub mod bech32;
#[cfg(feature = "encryption")]
pub mod encryption;
#[cfg(feature = "hashing")]
pub mod hashing;
// #[cfg(feature = "nft")]
// pub mod nft;
