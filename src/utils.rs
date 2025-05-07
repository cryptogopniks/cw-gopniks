use std::{collections::HashSet, hash::Hash};

use crate::cosmwasm_std::StdError;

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
