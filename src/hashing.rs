use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use cosmwasm_std::StdResult;

use crate::private_communication::convert_err;
pub use crate::private_communication::{Hash, ENC_KEY_LEN};

/// Accepts `password` as string of letters and numbers and `salt` as string of letters and numbers
pub fn calc_hash_bytes(password: &str, salt: &str) -> StdResult<[u8; ENC_KEY_LEN]> {
    const MEMORY_SIZE: u32 = 64;
    const NUMBER_OF_ITERATIONS: u32 = 4;
    const DEGREE_OF_PARALLELISM: u32 = 1;
    const ALGORITHM: Algorithm = Algorithm::Argon2id;
    const VERSION: Version = Version::V0x10;

    let params = ParamsBuilder::new()
        .m_cost(MEMORY_SIZE)
        .t_cost(NUMBER_OF_ITERATIONS)
        .p_cost(DEGREE_OF_PARALLELISM)
        .output_len(ENC_KEY_LEN)
        .build()
        .map_err(convert_err)?;

    let ctx = Argon2::new(ALGORITHM, VERSION, params);

    let mut out = [0; ENC_KEY_LEN];
    ctx.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut out)
        .map_err(convert_err)?;

    Ok(out)
}

pub fn address_to_salt(address: impl ToString) -> String {
    // Salt length must be >= 12
    address.to_string().repeat(2)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn default_hashing() {
        const PASSWORD: &str = "wasm18tnvnwkklyv4dyuj8x357n7vray4v4zur4crdj";
        const SALT: &str = "16898739935670952395686488112";

        const HASH_BYTES: [u8; 32] = [
            29, 244, 252, 166, 105, 232, 244, 214, 91, 151, 71, 223, 4, 50, 225, 64, 35, 214, 21,
            191, 196, 41, 144, 25, 192, 29, 99, 168, 195, 10, 205, 163,
        ];

        let hash = calc_hash_bytes(PASSWORD, SALT).unwrap();

        assert_eq!(hash, HASH_BYTES);
    }
}
