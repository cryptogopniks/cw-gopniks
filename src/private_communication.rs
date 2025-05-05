use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_hex, to_hex, Decimal, Env, StdError, StdResult, Timestamp};

pub const ENC_KEY_LEN: usize = 32;

#[cw_serde]
pub struct EncryptedResponse {
    pub value: String,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct ExecuteMsgWithTimestamp<T: Clone> {
    pub msg: T,
    pub timestamp: Timestamp,
}

impl<T: Clone> ExecuteMsgWithTimestamp<T> {
    pub fn new(env: &Env, msg: &T) -> Self {
        Self {
            msg: msg.to_owned(),
            timestamp: env.block.time,
        }
    }
}

#[cw_serde]
pub struct Hash {
    bytes: [u8; ENC_KEY_LEN],
}

impl Hash {
    pub fn parse(hex_str: &str) -> StdResult<Self> {
        u8_vec_to_hash_bytes(&from_hex(hex_str)?).map(|bytes| Self { bytes })
    }

    pub fn to_norm_dec(&self) -> Decimal {
        hash_bytes_to_norm_dec(&self.bytes)
    }
}

impl From<[u8; ENC_KEY_LEN]> for Hash {
    fn from(bytes: [u8; ENC_KEY_LEN]) -> Self {
        Self { bytes }
    }
}

impl From<Hash> for [u8; ENC_KEY_LEN] {
    fn from(hash: Hash) -> Self {
        hash.bytes
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", to_hex(self.bytes))
    }
}

/// Converts u8 vector to [u8; ENC_KEY_LEN]
fn u8_vec_to_hash_bytes(v: &[u8]) -> StdResult<[u8; ENC_KEY_LEN]> {
    TryInto::try_into(v.to_owned()).map_err(|_| {
        StdError::generic_err(format!(
            "Vector length is {} but expected {}",
            v.len(),
            ENC_KEY_LEN
        ))
    })
}

/// Converts [u8; ENC_KEY_LEN] to Decimal in range 0..1
fn hash_bytes_to_norm_dec(hash: &[u8; ENC_KEY_LEN]) -> Decimal {
    let mut hash_value: u128 = 0;

    for &byte in hash.iter().rev() {
        hash_value = (hash_value << 8) | (byte as u128);
    }

    Decimal::from_ratio(hash_value, u128::MAX)
}

pub fn convert_err(e: impl ToString) -> StdError {
    StdError::generic_err(e.to_string())
}
