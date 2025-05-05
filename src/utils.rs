use cosmwasm_std::StdError;

/// Converts an error to a StdError with the error message
pub fn convert_err(e: impl ToString) -> StdError {
    StdError::generic_err(e.to_string())
}
