//! # cw-bech32
//!
//! Helper utilities for working with bech32 addresses in CosmWasm contracts.
//!
//! This crate provides simple utilities to:
//! - Split and join bech32 addresses
//! - Convert between different bech32 prefix formats
//! - Handle bech32 addresses in a type-safe manner

use crate::cosmwasm_std::{Addr, StdError, StdResult};
use bech32::{decode, encode, Variant};

use crate::utils::convert_err;

/// The standard delimiter used in bech32 addresses
pub const BECH32_ADDR_DELIMITER: &str = "1";
/// Error message for invalid bech32 address format
pub const SPLIT_ERROR: &str = "Invalid bech32 address!";

/// Utility struct for working with bech32 addresses
pub struct Bech32Addr {}

impl Bech32Addr {
    /// Splits a bech32 address into its prefix and postfix components
    ///
    /// # Arguments
    ///
    /// * `address` - A bech32 address like "cosmos1..."
    ///
    /// # Returns
    ///
    /// A tuple containing the prefix and postfix of the address
    ///
    /// # Examples
    ///
    /// ```
    /// use cw_bech32::Bech32Addr;
    ///
    /// let address = "cosmos1abcdef...";
    /// let (prefix, postfix) = Bech32Addr::split(address).unwrap();
    /// assert_eq!(prefix, "cosmos");
    /// ```
    pub fn split(address: impl ToString) -> StdResult<(String, String)> {
        let address = address.to_string();
        let (prefix, postfix) = address
            .split_once(BECH32_ADDR_DELIMITER)
            .ok_or(StdError::generic_err(SPLIT_ERROR))
            .map_err(convert_err)?;

        Ok((prefix.to_string(), postfix.to_string()))
    }

    /// Joins a prefix and postfix to create a bech32 address
    ///
    /// # Arguments
    ///
    /// * `prefix` - The bech32 prefix (e.g., "cosmos", "osmo")
    /// * `postfix` - The postfix data part of the address
    ///
    /// # Returns
    ///
    /// A complete bech32 address string
    ///
    /// # Examples
    ///
    /// ```
    /// use cw_bech32::Bech32Addr;
    ///
    /// let address = Bech32Addr::join("osmo", "abcdef...");
    /// assert_eq!(address, "osmo1abcdef...");
    /// ```
    pub fn join(prefix: &str, postfix: &str) -> String {
        format!("{}{}{}", prefix, BECH32_ADDR_DELIMITER, postfix)
    }

    /// Converts a bech32 address to use a different prefix
    ///
    /// # Arguments
    ///
    /// * `address` - The original bech32 address
    /// * `prefix` - The new prefix to use
    ///
    /// # Returns
    ///
    /// A new bech32 address with the specified prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use cw_bech32::Bech32Addr;
    ///
    /// // Convert a Cosmos address to an Osmosis address
    /// let cosmos_addr = "cosmos1..."; // Replace with valid bech32 address for testing
    /// let osmo_addr = Bech32Addr::convert(cosmos_addr, "osmo").unwrap();
    /// ```
    pub fn convert(address: impl ToString, prefix: &str) -> StdResult<String> {
        let (_hrp, data, _) = decode(&address.to_string()).map_err(convert_err)?;
        encode(prefix, data, Variant::Bech32).map_err(convert_err)
    }
}

/// Trait for types that can be converted to a bech32 address with a given prefix
pub trait WithBech32 {
    /// Convert to an `Addr` type with the specified bech32 prefix
    ///
    /// # Arguments
    ///
    /// * `prefix` - The bech32 prefix to use
    ///
    /// # Returns
    ///
    /// An `Addr` with the specified prefix
    fn get(&self, prefix: &str) -> StdResult<Addr>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_join() -> StdResult<()> {
        const ADDRESS_COSMOS: &str = "cosmos1f37v0rdvrred27tlqqcpkrqpzfv6ddr2feflfd";
        const PREFIX_COSMOS: &str = "cosmos";
        const POSTFIX_COSMOS: &str = "f37v0rdvrred27tlqqcpkrqpzfv6ddr2feflfd";

        let (prefix, postfix) = Bech32Addr::split(ADDRESS_COSMOS)?;
        assert_eq!(prefix, PREFIX_COSMOS);
        assert_eq!(postfix, POSTFIX_COSMOS);

        let rejoined = Bech32Addr::join(&prefix, &postfix);
        assert_eq!(rejoined, ADDRESS_COSMOS);

        Ok(())
    }

    #[test]
    fn test_convert() -> StdResult<()> {
        const ADDRESS_COSMOS: &str = "cosmos1f37v0rdvrred27tlqqcpkrqpzfv6ddr2feflfd";
        const ADDRESS_OSMOSIS: &str = "osmo1f37v0rdvrred27tlqqcpkrqpzfv6ddr2pz60ll";
        const PREFIX_COSMOS: &str = "cosmos";
        const PREFIX_OSMOSIS: &str = "osmo";

        let address_osmosis = Bech32Addr::convert(ADDRESS_COSMOS, PREFIX_OSMOSIS)?;
        assert_eq!(address_osmosis, ADDRESS_OSMOSIS);

        let address_cosmos = Bech32Addr::convert(address_osmosis, PREFIX_COSMOS)?;
        assert_eq!(address_cosmos, ADDRESS_COSMOS);

        Ok(())
    }
}
