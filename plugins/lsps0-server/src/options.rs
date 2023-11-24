use cln_plugin::options::{ConfigOption, Value};

// TODO: All these options should become dynamic
//
// 1. Adapt cln_plugin to allow for dynamic config options
// 2. Implement it here
//
// TODO: We should find a way to validate the options
// value. Currently, the server-admin should ensure
// the options are sensible and spec-compliant.

pub fn lsps1_info_website() -> ConfigOption {
    ConfigOption::new(
        "lsps1_info_website",
        Value::OptString,
        "The website advertised in LSPS1",
    )
}

pub fn lsps1_minimum_channel_confirmations() -> ConfigOption {
    ConfigOption::new(
        "lsps1_minimum_channel_confirmations",
        Value::Integer(6),
        "Minimum number of block confirmations before the LSP accepts a channel as confirmed and sends channel_ready"
        )
}

pub fn lsps1_minimum_onchain_payment_confirmations() -> ConfigOption {
    ConfigOption::new(
        "lsps1_minimum_onchain_payment_confirmations",
        Value::OptInteger,
        "Minimum number of block confirmations before the LSP accepts an on-chain payment as confirmed. This is a lower bound."
    )
}

pub fn lsps1_supports_zero_channel_reserve() -> ConfigOption {
    ConfigOption::new(
        "lsps1_supports_zero_channel_reserve",
        Value::Boolean(false),
        "Indicates if the LSP supports zeroreserve",
    )
}

pub fn lsps1_max_channel_expiry_blocks() -> ConfigOption {
    ConfigOption::new(
        "lsps1_max_channel_expiry_blocks",
        Value::Integer(51260),
        "The maximum number of blocks a channel can be leased for. ",
    )
}

pub fn lsps1_min_onchain_payment_size_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_min_onchain_payment_size_sat",
        Value::OptString,
        "Indicates the minimum amount of satoshi (order_total_sat) that is required for the LSP to accept a payment on-chain"
    )
}

pub fn lsps1_min_initial_client_balance_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_min_initial_client_balance_sat",
        Value::OptString,
        "Minimum number of satoshi that the client MUST request.",
    )
}

pub fn lsps1_max_initial_client_balance_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_max_initial_client_balance_sat",
        Value::OptString,
        "Maximum number of satoshi that the client MUST request.",
    )
}

pub fn lsps1_min_initial_lsp_balance_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_min_initial_lsp_balance_sat",
        Value::OptString,
        "Minimum number of satoshi that the LSP will provide to the channel.",
    )
}

pub fn lsps1_max_initial_lsp_balance_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_max_initial_lsp_balance_sat",
        Value::OptString,
        "Maximum number of satoshi that the LSP will provide to the channel.",
    )
}

pub fn lsps1_min_channel_balance_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_min_channel_balance_sat",
        Value::OptString,
        "Minimum channel size",
    )
}

pub fn lsps1_max_channel_balance_sat() -> ConfigOption {
    ConfigOption::new(
        "lsps1_max_channel_balance_sat",
        Value::OptString,
        "Maximum Channel size",
    )
}
