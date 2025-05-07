use crate::cosmwasm_std;

use cosmwasm_std::{coins, Addr, Binary, Coin, CosmosMsg, Uint128};

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
    use super::*;

    const PORT_DEFAULT: &str = "transfer";

    pub mod regular {
        use super::*;

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
        use super::*;

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
