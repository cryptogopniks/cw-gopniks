use cosmwasm_std::{from_base64, from_json, to_base64, to_json_vec, StdResult, Timestamp};
use serde::{de::DeserializeOwned, Serialize};

pub use crate::private_communication::{
    EncryptedResponse, ExecuteMsgWithTimestamp, Hash, ENC_KEY_LEN,
};
use crate::utils::convert_err;

use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};

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
