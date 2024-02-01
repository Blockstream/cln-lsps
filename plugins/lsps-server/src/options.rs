use cln_plugin::options::{ConfigOption, Value};

// TODO: All these options should become dynamic
//
// 1. Adapt cln_plugin to allow for dynamic config options
// 2. Implement it here
//
// TODO: We should find a way to validate the options
// value. Currently, the server-admin should ensure
// the options are sensible and spec-compliant.

pub(crate) const LSPS1_ENABLE: &str = "lsps1-enable";
pub(crate) const LSPS1_INFO_WEBSITE: &str = "lsps1-info-website";
pub(crate) const LSPS1_MIN_CHANNEL_CONFIRMATIONS: &str = "lsps1-minimum-channel-confirmations";
pub(crate) const LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS: &str =
    "lsps1-minimum-onchain-payment-confirmations";
pub(crate) const LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE: &str = "lsps1-supports-zero-channel-reserve";
pub(crate) const LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS: &str = "lsps1-max-channel-expiry-blocks";
pub(crate) const LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT: &str = "lsps1-min-onchain-payment-size-sat";

pub(crate) const LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT: &str =
    "lsps1-min-initial-client-balance-sat";
pub(crate) const LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT: &str =
    "lsps1-max-initial-client-balance-sat";
pub(crate) const LSPS1_MIN_INITIAL_LSP_BALANCE_SAT: &str = "lsps1-min-initial-lsp-balance-sat";
pub(crate) const LSPS1_MAX_INITIAL_LSP_BALANCE_SAT: &str = "lsps1-max-initial-lsp-balance-sat";
pub(crate) const LSPS1_MIN_CHANNEL_BALANCE_SAT: &str = "lsps1-min-channel-balance-sat";
pub(crate) const LSPS1_MAX_CHANNEL_BALANCE_SAT: &str = "lsps1-max-channel-balance-sat";

pub(crate) const LSPS1_ORDER_LIFETIME: &str = "lsps1-order-lifetime";
pub(crate) const LSPS1_FEE_COMPUTATION_BASE_FEE_SAT: &str = "lsps1-fee-computation-base-fee-sat";
pub(crate) const LSPS1_FEE_COMPUTATION_WEIGHT_UNITS: &str = "lsps1-fee-computation-weight-units";
pub(crate) const LSPS1_FEE_COMPUTATION_LIQUIDITY_PPB: &str = "lsps1-fee-computation-liquidity-ppb";
pub(crate) const LSP_SERVER_DATABASE_URL: &str = "lsp-server-database-url";

pub fn lsps1_enable() -> ConfigOption {
    ConfigOption::new(
        LSPS1_ENABLE,
        Value::Boolean(false),
        "Set to true to enable LSPS1",
    )
}

pub fn lsps1_info_website() -> ConfigOption {
    ConfigOption::new(
        LSPS1_INFO_WEBSITE,
        Value::OptString,
        "The website advertised in LSPS1",
    )
}

pub fn lsps1_minimum_channel_confirmations() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MIN_CHANNEL_CONFIRMATIONS,
        Value::Integer(6),
        "Minimum number of block confirmations before the LSP accepts a channel as confirmed and sends channel_ready"
        )
}

pub fn lsps1_minimum_onchain_payment_confirmations() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS,
        Value::Integer(6),
        "Minimum number of block confirmations before the LSP accepts an on-chain payment as confirmed. This is a lower bound."
    )
}

pub fn lsps1_supports_zero_channel_reserve() -> ConfigOption {
    ConfigOption::new(
        LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE,
        Value::Boolean(false),
        "Indicates if the LSP supports zeroreserve",
    )
}

pub fn lsps1_max_channel_expiry_blocks() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS,
        Value::Integer(5000),
        "The maximum number of blocks a channel can be leased for. (Default is 5000 blocks. This is a bit over 1 month) ",
    )
}

pub fn lsps1_min_onchain_payment_size_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT,
        Value::OptString,
        "Indicates the minimum amount of satoshi (order_total_sat) that is required for the LSP to accept a payment on-chain"
    )
}

pub fn lsps1_fee_computation_base_fee_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_FEE_COMPUTATION_BASE_FEE_SAT,
        Value::Integer(100),
        "Base fee used for onchain cost for fee-computation LSPS1",
    )
}

pub fn lsps1_fee_computation_onchain_ppm() -> ConfigOption {
    ConfigOption::new(
        LSPS1_FEE_COMPUTATION_WEIGHT_UNITS,
        Value::Integer(500),
        "Multiplier used for onchain-cost for fee-computation LSPS1",
    )
}

pub fn lsps1_fee_computation_liquidity_ppb() -> ConfigOption {
    ConfigOption::new(
        LSPS1_FEE_COMPUTATION_LIQUIDITY_PPB,
        Value::Integer(200),
        "Multiplier used to represent liquidity cost for fee-computaiton LSPS1",
    )
}

pub fn lsps1_order_lifetime_seconds() -> ConfigOption {
    ConfigOption::new(
        LSPS1_ORDER_LIFETIME,
        Value::Integer(3600 * 6),
        "The amount of seconds an order is deemd valid (Default is 6 hours)",
    )
}

pub fn lsp_server_database_url() -> ConfigOption {
    ConfigOption::new(
        LSP_SERVER_DATABASE_URL,
        Value::OptString,
        "The fully qualfied patth to the database. E.g: sqlite://home/user/data/lsp_server_database.db")
}

pub fn lsps1_min_initial_client_balance_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT,
        Value::OptString,
        "Minimum number of satoshis the client can request",
    )
}

pub fn lsps1_max_initial_client_balance_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT,
        Value::OptString,
        "Maximum number of satoshis the client can request",
    )
}

pub fn lsps1_min_initial_lsp_balance_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MIN_INITIAL_LSP_BALANCE_SAT,
        Value::OptString,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_max_initial_lsp_balance_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MAX_INITIAL_LSP_BALANCE_SAT,
        Value::OptString,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_min_channel_balance_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MIN_CHANNEL_BALANCE_SAT,
        Value::OptString,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_max_channel_balance_sat() -> ConfigOption {
    ConfigOption::new(
        LSPS1_MAX_CHANNEL_BALANCE_SAT,
        Value::OptString,
        "Minimum numbers of satoshis the server will provide",
    )
}
