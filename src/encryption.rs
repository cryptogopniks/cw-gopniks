use crate::cosmwasm_std;

use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
use cosmwasm_std::{from_json, to_json_vec, StdResult, Timestamp};

#[cfg(feature = "cw-v2")]
use crate::cosmwasm_std::{from_base64, to_base64};

use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};

pub use crate::private_communication::{
    EncryptedResponse, ExecuteMsgWithTimestamp, Hash, ENC_KEY_LEN,
};
use crate::utils::convert_err;

/// Base64 encoding engine used in conversion to/from base64.
///
/// The engine adds padding when encoding and accepts strings with or
/// without padding when decoding.
#[cfg(feature = "cw-v1")]
const B64_ENGINE: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::GeneralPurposeConfig::new()
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent),
);

/// Deserialize a bag of bytes from Base64 into a vector of bytes
#[cfg(feature = "cw-v1")]
fn from_base64<I>(input: I) -> StdResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    base64::Engine::decode(&B64_ENGINE, input).map_err(convert_err)
}

/// Encode a bag of bytes into the Base64 format
#[cfg(feature = "cw-v1")]
fn to_base64<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    base64::Engine::encode(&B64_ENGINE, input)
}

fn timestamp_to_nonce(timestamp: &Timestamp) -> String {
    // Nonce length must be 12
    timestamp.nanos().to_string()[..12].to_string()
}

fn get_cipher(enc_key: &[u8; ENC_KEY_LEN]) -> StdResult<Aes256GcmSiv> {
    let generic_array: &GenericArray<u8, _> = &GenericArray::from(*enc_key);

    Ok(Aes256GcmSiv::new(generic_array))
}

fn encrypt(msg: &[u8], enc_key: &[u8; ENC_KEY_LEN], nonce: &str) -> StdResult<String> {
    let nonce: &GenericArray<u8, _> = Nonce::from_slice(nonce.as_bytes());
    let cipher = get_cipher(enc_key)?;

    cipher
        .encrypt(nonce, msg)
        .map(to_base64)
        .map_err(convert_err)
}

fn decrypt(enc_msg: &str, enc_key: &[u8; ENC_KEY_LEN], nonce: &str) -> StdResult<Vec<u8>> {
    let nonce: &GenericArray<u8, _> = Nonce::from_slice(nonce.as_bytes());
    let cipher = get_cipher(enc_key)?;

    cipher
        .decrypt(nonce, &from_base64(enc_msg)?[..])
        .map_err(convert_err)
}

fn serialize<T: ?Sized + Serialize>(data: &T) -> StdResult<Vec<u8>> {
    to_json_vec(data)
}

fn deserialize<T: DeserializeOwned>(data: &[u8]) -> StdResult<T> {
    from_json::<T>(data)
}

pub fn decrypt_deserialize<T, F>(enc_key: &F, timestamp: &Timestamp, value: &str) -> StdResult<T>
where
    T: DeserializeOwned,
    F: Into<[u8; ENC_KEY_LEN]> + Clone,
{
    let encryption_key = &enc_key.to_owned().into();
    let nonce = &timestamp_to_nonce(timestamp);
    let decrypted_data = &decrypt(value, encryption_key, nonce)?;

    deserialize(decrypted_data)
}

pub fn serialize_encrypt<T, F>(
    enc_key: &F,
    timestamp: &Timestamp,
    value: &T,
) -> StdResult<EncryptedResponse>
where
    T: ?Sized + Serialize,
    F: Into<[u8; ENC_KEY_LEN]> + Clone,
{
    let encryption_key = &enc_key.to_owned().into();
    let nonce = &timestamp_to_nonce(timestamp);
    let serialized_value = &serialize(value)?;
    let encrypted_value = encrypt(serialized_value, encryption_key, nonce)?;

    Ok(EncryptedResponse {
        value: encrypted_value,
        timestamp: timestamp.to_owned(),
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn default_ecryption() -> StdResult<()> {
        const MESSAGE: &str = "The secret message #1. Don't share it!⚠️";
        const ENC_KEY: &[u8; 32] = &[1; 32];
        const NONCE: &str = "unique nonce";

        let encrypted = encrypt(&serialize(MESSAGE)?, ENC_KEY, NONCE)?;
        let decrypted: String = deserialize(&decrypt(&encrypted, ENC_KEY, NONCE)?)?;

        assert_ne!(encrypted, decrypted);
        assert_eq!(MESSAGE, decrypted);

        Ok(())
    }
}
