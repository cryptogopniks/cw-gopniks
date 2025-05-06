#[cfg(all(feature = "cw-std-v1", feature = "cw-std-v2"))]
compile_error!("Features `cw-std-v1` and `cw-std-v2` are mutually exclusive. Enable only one");

mod utils;

// #[cfg(any(feature = "encryption", feature = "hashing"))]
// mod private_communication;

#[cfg(any(feature = "cw-std-v1", feature = "cw-std-v2"))]
pub mod cw;

// #[cfg(feature = "assets")]
// pub mod assets;
// #[cfg(feature = "auth")]
// pub mod auth;
// #[cfg(feature = "bech32")]
// pub mod bech32;
// #[cfg(feature = "encryption")]
// pub mod encryption;
// #[cfg(feature = "hashing")]
// pub mod hashing;
// #[cfg(feature = "nft")]
// pub mod nft;
