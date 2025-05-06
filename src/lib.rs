#[cfg(all(not(feature = "cw-std-v1"), not(feature = "cw-std-v2")))]
compile_error!("`cw-std-v1` or `cw-std-v2` feature is required");
#[cfg(all(feature = "cw-std-v1", feature = "cw-std-v2"))]
compile_error!("Features `cw-std-v1` and `cw-std-v2` are mutually exclusive. Enable only one");

#[cfg(all(feature = "cw-std-v1", not(feature = "cw-std-v2")))]
use cosmwasm_std_v1 as cosmwasm_std;
#[cfg(all(feature = "cw-std-v2", not(feature = "cw-std-v1")))]
use cosmwasm_std_v2 as cosmwasm_std;

mod utils;

#[cfg(any(feature = "encryption", feature = "hashing"))]
mod private_communication;

#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "auth")]
pub mod auth;
#[cfg(feature = "bech32")]
pub mod bech32;
#[cfg(feature = "encryption")]
pub mod encryption;
#[cfg(feature = "hashing")]
pub mod hashing;
#[cfg(feature = "nft")]
pub mod nft;
