use cln_plugin::options;

// TODO: All these options should become dynamic
//
// 1. Adapt cln_plugin to allow for dynamic config options
// 2. Implement it here
//
// TODO: We should find a way to validate the options
// value. Currently, the server-admin should ensure
// the options are sensible and spec-compliant.

pub(crate) const LSPS1_ENABLE: &str = "lsps1-enable";
pub(crate) const LSPS1_MIN_CHANNEL_CONFIRMATIONS: &str = "lsps1-min-required-channel-confirmations";
pub(crate) const LSPS1_MIN_FUNDING_CONFIRMS_WITHIN_BLOCKS: &str =
    "lsps1-min-funding-confirms-within-blocks";
pub(crate) const LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS: &str =
    "lsps1-min-onchain-payment-confirmations";
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

pub fn lsps1_enable() -> options::FlagConfigOption<'static> {
    options::FlagConfigOption::new_flag(LSPS1_ENABLE, "If set LSPS1 is enabled")
}

pub fn lsps1_min_required_channel_confirmations() -> options::DefaultIntegerConfigOption<'static> {
    options::DefaultIntegerConfigOption::new_i64_with_default(
        LSPS1_MIN_CHANNEL_CONFIRMATIONS,
        6,
        "Minimum number of block confirmations before the LSP accepts a channel as confirmed and sends channel_ready"
        )
}

pub fn lsps1_min_onchain_payment_confirmations() -> options::IntegerConfigOption<'static> {
    options::IntegerConfigOption::new_i64_no_default(
        LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS,
        "Minimum number of block confirmations before the LSP accepts an on-chain payment as confirmed. This is a lower bound."
    )
}

pub fn lsps1_supports_zero_channel_reserve() -> options::DefaultBooleanConfigOption<'static> {
    options::DefaultBooleanConfigOption::new_bool_with_default(
        LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE,
        false,
        "Indicates if the LSP supports zeroreserve",
    )
}

pub fn lsps1_max_channel_expiry_blocks() -> options::DefaultIntegerConfigOption<'static> {
    options::DefaultIntegerConfigOption::new_i64_with_default(
        LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS,
        5000,
        "The maximum number of blocks a channel can be leased for. (Default is 5000 blocks. This is a bit over 1 month) ",
    )
}

pub fn lsps1_min_onchain_payment_size_sat() -> options::IntegerConfigOption<'static> {
    options::IntegerConfigOption::new_i64_no_default(
        LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT,
        "Indicates the minimum amount of satoshi (order_total_sat) that is required for the LSP to accept a payment on-chain"
    )
}

pub fn lsps1_fee_computation_base_fee_sat() -> options::DefaultIntegerConfigOption<'static> {
    options::DefaultIntegerConfigOption::new_i64_with_default(
        LSPS1_FEE_COMPUTATION_BASE_FEE_SAT,
        100,
        "Base fee used for onchain cost for fee-computation LSPS1",
    )
}

pub fn lsps1_fee_computation_onchain_ppm() -> options::DefaultIntegerConfigOption<'static> {
    options::DefaultIntegerConfigOption::new_i64_with_default(
        LSPS1_FEE_COMPUTATION_WEIGHT_UNITS,
        500,
        "Multiplier used for onchain-cost for fee-computation LSPS1",
    )
}

pub fn lsps1_fee_computation_liquidity_ppb() -> options::DefaultIntegerConfigOption<'static> {
    options::DefaultIntegerConfigOption::new_i64_with_default(
        LSPS1_FEE_COMPUTATION_LIQUIDITY_PPB,
        200,
        "Multiplier used to represent liquidity cost for fee-computaiton LSPS1",
    )
}

pub fn lsps1_order_lifetime_seconds() -> options::DefaultIntegerConfigOption<'static> {
    options::ConfigOption::new_i64_with_default(
        LSPS1_ORDER_LIFETIME,
        3600 * 6,
        "The amount of seconds an order is deemd valid (Default is 6 hours)",
    )
}

pub fn lsp_server_database_url() -> options::StringConfigOption<'static> {
    options::StringConfigOption::new_str_no_default(
        LSP_SERVER_DATABASE_URL,
        "The fully qualfied patth to the database. E.g: sqlite://home/user/data/lsp_server_database.db")
}

pub fn lsps1_min_initial_client_balance_sat() -> options::IntegerConfigOption<'static> {
    options::ConfigOption::new_i64_no_default(
        LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT,
        "Minimum number of satoshis the client can request",
    )
}

pub fn lsps1_max_initial_client_balance_sat() -> options::IntegerConfigOption<'static> {
    options::ConfigOption::new_i64_no_default(
        LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT,
        "Maximum number of satoshis the client can request",
    )
}

pub fn lsps1_min_initial_lsp_balance_sat() -> options::IntegerConfigOption<'static> {
    options::IntegerConfigOption::new_i64_no_default(
        LSPS1_MIN_INITIAL_LSP_BALANCE_SAT,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_max_initial_lsp_balance_sat() -> options::IntegerConfigOption<'static> {
    options::IntegerConfigOption::new_i64_no_default(
        LSPS1_MAX_INITIAL_LSP_BALANCE_SAT,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_min_channel_balance_sat() -> options::IntegerConfigOption<'static> {
    options::IntegerConfigOption::new_i64_no_default(
        LSPS1_MIN_CHANNEL_BALANCE_SAT,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_max_channel_balance_sat() -> options::IntegerConfigOption<'static> {
    options::IntegerConfigOption::new_i64_no_default(
        LSPS1_MAX_CHANNEL_BALANCE_SAT,
        "Minimum numbers of satoshis the server will provide",
    )
}

pub fn lsps1_min_funding_confirms_within_blocks() -> options::DefaultIntegerConfigOption<'static> {
    options::DefaultIntegerConfigOption::new_i64_with_default(
        LSPS1_MIN_FUNDING_CONFIRMS_WITHIN_BLOCKS,
        0,
        "The fastest speed at which the LSP can confirm a block",
    )
}
