use crate::cosmwasm_std;

use cosmwasm_std::{Binary, Coin, CosmosMsg};

use anybuf::Anybuf;

fn get_any_msg(type_url: &str, value: Binary) -> CosmosMsg {
    #[cfg(feature = "cw-v1")]
    let msg = CosmosMsg::Stargate {
        type_url: type_url.to_string(),
        value,
    };

    #[cfg(feature = "cw-v2")]
    let msg = CosmosMsg::Any(cosmwasm_std::AnyMsg {
        type_url: type_url.to_string(),
        value,
    });

    msg
}

fn get_coin_msgs(coin_list: &[Coin]) -> Vec<Anybuf> {
    coin_list
        .iter()
        .map(|coin| {
            Anybuf::new()
                .append_string(1, coin.denom.clone())
                .append_string(2, coin.amount.to_string())
        })
        .collect()
}

pub mod ibc {
    use cosmwasm_schema::cw_serde;

    const PORT_DEFAULT: &str = "transfer";

    #[cw_serde]
    pub enum IbcMemo<M> {
        Forward {
            channel: String,
            port: String,
            receiver: String,
            retries: u8,
            timeout: u64,
        },
        Wasm {
            contract: String,
            msg: M,
        },
    }

    pub mod regular {
        use crate::{
            any::{get_any_msg, get_coin_msgs, ibc::PORT_DEFAULT},
            cosmwasm_std::{coins, Addr, CosmosMsg, Uint128},
        };
        use anybuf::Anybuf;

        #[allow(clippy::too_many_arguments)]
        pub fn get_transfer_msg(
            port: Option<&str>,
            channel: &str,
            denom_in: &str,
            amount_in: Uint128,
            sender: &Addr,
            contract_address: &str,
            timeout_timestamp_ns: u64,
            ibc_transfer_memo: &str,
        ) -> CosmosMsg {
            // https://github.com/osmosis-labs/osmosis/blob/main/cosmwasm/packages/registry/src/proto.rs#L32
            get_any_msg(
                "/ibc.applications.transfer.v1.MsgTransfer",
                Anybuf::new()
                    // source port
                    .append_string(1, port.unwrap_or(PORT_DEFAULT))
                    // source channel (IBC Channel on your network side)
                    .append_string(2, channel)
                    // token
                    .append_message(
                        3,
                        get_coin_msgs(&coins(amount_in.u128(), denom_in))
                            .first()
                            .unwrap(),
                    )
                    // sender
                    .append_string(4, sender)
                    // recipient
                    .append_string(5, contract_address)
                    // TimeoutHeight
                    .append_message(6, &Anybuf::new().append_uint64(1, 0).append_uint64(2, 0))
                    // TimeoutTimestamp
                    .append_uint64(7, timeout_timestamp_ns)
                    // IBC Hook memo
                    .append_string(8, ibc_transfer_memo)
                    .into_vec()
                    .into(),
            )
        }
    }

    pub mod neutron {
        use crate::{
            any::{get_any_msg, get_coin_msgs, ibc::PORT_DEFAULT},
            cosmwasm_std::{coins, Addr, Binary, Coin, CosmosMsg, Uint128},
        };
        use anybuf::Anybuf;
        use cosmwasm_schema::cw_serde;

        // https://github.com/neutron-org/neutron-sdk/blob/main/packages/neutron-sdk/src/sudo/msg.rs
        #[cw_serde]
        pub struct RequestPacket {
            pub sequence: Option<u64>,
            pub source_port: Option<String>,
            pub source_channel: Option<String>,
            pub destination_port: Option<String>,
            pub destination_channel: Option<String>,
            pub data: Option<Binary>,
            pub timeout_height: Option<RequestPacketTimeoutHeight>,
            pub timeout_timestamp: Option<u64>,
        }

        #[cw_serde]
        pub struct RequestPacketTimeoutHeight {
            pub revision_number: Option<u64>,
            pub revision_height: Option<u64>,
        }

        /// Height is used for sudo call for `TxQueryResult` enum variant type
        #[cw_serde]
        pub struct Height {
            /// the revision that the client is currently on
            #[serde(default)]
            pub revision_number: u64,
            /// **height** is a height of remote chain
            #[serde(default)]
            pub revision_height: u64,
        }

        // https://github.com/neutron-org/neutron-sdk/blob/main/packages/neutron-sdk/src/sudo/msg.rs
        #[cw_serde]
        pub enum SudoMsg {
            Response {
                request: RequestPacket,
                data: Binary,
            },
            Error {
                request: RequestPacket,
                details: String,
            },
            Timeout {
                request: RequestPacket,
            },
            OpenAck {
                port_id: String,
                channel_id: String,
                counterparty_channel_id: String,
                counterparty_version: String,
            },
            TxQueryResult {
                query_id: u64,
                height: Height,
                data: Binary,
            },
            #[serde(rename = "kv_query_result")]
            KVQueryResult {
                query_id: u64,
            },
        }

        #[allow(clippy::too_many_arguments)]
        pub fn get_transfer_msg(
            port: Option<&str>,
            channel: &str,
            denom_in: &str,
            amount_in: Uint128,
            sender: &Addr,
            contract_address: &str,
            timeout_timestamp_ns: u64,
            ibc_transfer_memo: &str,
            refunder_fee: &[Coin],
        ) -> CosmosMsg {
            let recv_fee: &Vec<Coin> = &vec![];
            let ack_fee = refunder_fee;
            let timeout_fee = refunder_fee;

            // https://github.com/neutron-org/neutron-std/blob/main/packages/neutron-std/src/types/neutron/transfer.rs
            // https://github.com/neutron-org/neutron/blob/main/proto/neutron/transfer/v1/tx.proto#L25
            get_any_msg(
                "/neutron.transfer.MsgTransfer",
                Anybuf::new()
                    // source port
                    .append_string(1, port.unwrap_or(PORT_DEFAULT))
                    // source channel (IBC Channel on your network side)
                    .append_string(2, channel)
                    // token
                    .append_message(
                        3,
                        get_coin_msgs(&coins(amount_in.u128(), denom_in))
                            .first()
                            .unwrap(),
                    )
                    // sender
                    .append_string(4, sender)
                    // recipient
                    .append_string(5, contract_address)
                    // TimeoutHeight
                    .append_message(6, &Anybuf::new().append_uint64(1, 0).append_uint64(2, 0))
                    // TimeoutTimestamp
                    .append_uint64(7, timeout_timestamp_ns)
                    // IBC Hook memo
                    .append_string(8, ibc_transfer_memo)
                    // fee refunder
                    .append_message(
                        9,
                        &Anybuf::new()
                            .append_repeated_message(1, &get_coin_msgs(recv_fee))
                            .append_repeated_message(2, &get_coin_msgs(ack_fee))
                            .append_repeated_message(3, &get_coin_msgs(timeout_fee)),
                    )
                    .into_vec()
                    .into(),
            )
        }
    }
}
