# cw-gopniks

CosmWasm codebase used across (and beyond) CryptoGopniks projects

## Features

### - assets

##### Description

Helpers to work with native and cw20 tokens in CosmWasm contracts

##### Functionality

- Types for native and cw20 tokens
- Validation for native and cw20 tokens sent to a contract

##### Usage

```rust
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw_gopniks::assets::{Funds, InfoResp};

pub fn try_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let InfoResp {
            sender,
            asset_amount,
            asset_token,
        } = Funds::single(sender, amount).check(&deps.api, &info)?;

    // ...
}
```

### - auth

##### Description

Helpers to work with sender assertions and updating config admin in CosmWasm contracts

##### Functionality

- Assert single address, optional address, list of address in any combinations
- Transfer config admin safely

##### Usage

```rust
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use cw_gopniks::auth::{Auth, TransferAdminState};

pub fn try_update_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Auth::simple(&config.admin).assert(&info.sender)?;

    TransferAdminState::update_admin(
        deps,
        &env,
        &info.sender,
        &config.admin,
        &admin,
        TRANSFER_ADMIN_TIMEOUT,
    )?;

    // ...
}

pub fn try_accept_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let admin = TransferAdminState::accept_admin(deps, &env, &info.sender)?;

    CONFIG.update(deps.storage, |mut x| -> StdResult<_> {
        x.admin = admin;
        Ok(x)
    })?;

    // ...
}
```

### - bech32

##### Description

Helpers to handle bech32 addresses in CosmWasm smart contracts

##### Functionality

- Split and join bech32 addresses
- Convert addresses between different bech32 prefixes
- Trait for working with bech32 addresses

##### Usage

```rust
use cosmwasm_std::Addr;
use cw_gopniks::bech32::{Bech32Addr, WithBech32};

// Convert from one prefix to another
let cosmos_address = "cosmos1...";
let osmo_address = Bech32Addr::convert(cosmos_address, "osmo")?;

// Split a bech32 address into prefix and data parts
let (prefix, data) = Bech32Addr::split("cosmos1...")?;

// Join prefix and data into a bech32 address
let address = Bech32Addr::join("osmo", &data);
```

### - encryption

##### Description

Helpers to use aes-gcm-siv encryption in CosmWasm contracts

##### Functionality

- Serialize and encrypt data
- Decrypt and deserialize data

##### Usage

```rust
use cosmwasm_std::StdResult;
use cw_gopniks::encryption::{decrypt_deserialize, serialize_encrypt, EncryptedResponse, Hash};

const MESSAGE: &str = "The secret message";
/// 32 char hex encoded string
pub const ENC_KEY: &str =
	"0000000000000000000000000000000000000000000000000000000000000000";

let enc_key = Hash::parse(ENC_KEY)?;
let timestamp = env.block.time;
let encrypted_response = serialize_encrypt(&enc_key, &timestamp, MESSAGE)?;

let EncryptedResponse { value, timestamp } = encrypted_response;
let message: String = decrypt_deserialize(&enc_key, &timestamp, &value)?;
```

### - hashing

##### Description

Helpers to use argon2 hashing in CosmWasm contracts

##### Functionality

- Calculate hash bytes from password and salt
- Convert bech32 addresses to a salt
- Convert hex string to hash bytes
- Convert hash bytes to normalized decimal

##### Usage

```rust
use cosmwasm_std::{Addr, Decimal, Env, StdResult};
use cw_argon2::{address_to_salt, calc_hash_bytes, Hash};

pub fn get_random_weight(
    env: &Env,
    sender_address: &Addr,
    previous_weight: &Decimal,
) -> StdResult<Decimal> {
    let password = &format!("{}{}", previous_weight, env.block.time.nanos());
    let salt = &address_to_salt(sender_address);
    let hash_bytes = calc_hash_bytes(password, salt)?;

    Ok(Hash::from(hash_bytes).to_norm_dec())
}
```

### - nft

##### Description

Helpers to work with cw721-base in CosmWasm contracts

##### Functionality

- Version agnostic min set of Execute msgs for cw721-base

## Licenses

This repo is licensed under [Apache 2.0](./LICENSE).