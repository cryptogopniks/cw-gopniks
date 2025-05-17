use crate::cosmwasm_std;
use std::{collections::HashSet, hash::Hash};

use cosmwasm_std::{instantiate2_address, Addr, Binary, Deps, Env, StdError, StdResult};

/// converts an error to a StdError with the error message
pub fn convert_err(e: impl ToString) -> StdError {
    StdError::generic_err(e.to_string())
}

/// checks if a list has duplicates
pub fn has_duplicates<T: Eq + Hash>(list: &[T]) -> bool {
    let mut set = HashSet::with_capacity(list.len());

    for item in list {
        if !set.insert(item) {
            return true;
        }
    }

    false
}

/// removes duplicates from a list
pub fn deduplicate<T: Eq + Hash + Clone>(list: &[T]) -> Vec<T> {
    let mut set = HashSet::with_capacity(list.len());

    list.iter()
        .filter_map(|item| {
            if set.insert(item) {
                Some(item.to_owned())
            } else {
                None
            }
        })
        .collect()
}

// TODO: maybe move to nft or tests
pub fn to_string_vec<T: ToString>(vec: &[T]) -> Vec<String> {
    vec.iter().map(|x| x.to_string()).collect()
}

/// returns (address, salt)
#[cfg(any(feature = "hashing-v1", feature = "hashing-v2"))]
pub fn get_instantiate_2_addr(
    deps: Deps,
    env: &Env,
    label: &str,
    code_id: u64,
) -> StdResult<(Addr, Binary)> {
    let code_res = deps.querier.query_wasm_code_info(code_id)?;
    let salt = generate_salt(label, env)?;

    // predict the contract address
    let addr_raw = instantiate2_address(
        &code_res.checksum.as_slice(),
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        &salt,
    )
    .map_err(convert_err)?;
    let addr = deps.api.addr_humanize(&addr_raw)?;

    Ok((addr, salt))
}

#[cfg(any(feature = "hashing-v1", feature = "hashing-v2"))]
fn generate_salt(label: &str, env: &Env) -> StdResult<Binary> {
    let password = &format!("{}{}", label, env.block.time.nanos());
    let salt = &crate::hashing::address_to_salt(&env.contract.address);
    let hash_bytes = crate::hashing::calc_hash_bytes(password, salt)?;

    Ok(hash_bytes.to_vec().into())
}
