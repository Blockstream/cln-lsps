use anyhow::{anyhow, Context, Result};
use lsp_primitives::lsps1::schema::Lsps1Options;
use lsp_primitives::methods::Lsps1InfoResponse;
use std::convert::TryFrom;

use cln_plugin::options::Value;
use cln_plugin::ConfiguredPlugin;

use lsp_primitives::lsps0::common_schemas::SatAmount;
use lsp_primitives::lsps1::builders::{Lsps1InfoResponseBuilder, Lsps1OptionsBuilder};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::options::{
    LSPS1_ENABLE, LSPS1_INFO_WEBSITE, LSPS1_MAX_CHANNEL_BALANCE_SAT,
    LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS, LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT,
    LSPS1_MAX_INITIAL_LSP_BALANCE_SAT, LSPS1_MIN_CHANNEL_BALANCE_SAT,
    LSPS1_MIN_CHANNEL_CONFIRMATIONS, LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT,
    LSPS1_MIN_INITIAL_LSP_BALANCE_SAT, LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS,
    LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT, LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE,
};
use crate::state::PluginState;

fn parse_opt_str(value: cln_plugin::options::Value, name: &str) -> Result<Option<String>> {
    match value {
        Value::String(s) => Ok(Some(s)),
        Value::OptString => Ok(None),
        _ => Err(anyhow!("Configured value for '{}' is not a string", name)),
    }
}

fn parse_i64(value: cln_plugin::options::Value, name: &str) -> Result<i64> {
    let v = value.as_i64();

    match v {
        Some(v) => Ok(v),
        None => Err(anyhow!(
            "Invalid configuration {}: Expected number but was {:?}",
            name,
            value
        )),
    }
}

fn parse_bool(value: cln_plugin::options::Value, name: &str) -> Result<bool> {
    let v = value.as_bool();

    match v {
        Some(v) => Ok(v),
        None => Err(anyhow!(
            "Invalid configuration {}: Expected number but was {:?}",
            name,
            value
        )),
    }
}

fn parse_str(value: cln_plugin::options::Value, name: &str) -> Result<String> {
    let v = value.as_str();

    match v {
        Some(v) => Ok(v.to_string()),
        None => Err(anyhow!(
            "Invalid configuration {}: Expected number but was {:?}",
            name,
            value
        )),
    }
}

fn parse_num<T>(value: cln_plugin::options::Value, name: &str) -> Result<T>
where
    T: TryFrom<i64>,
{
    parse_i64(value, name).and_then(|n| {
        T::try_from(n).map_err(|_| {
            anyhow::anyhow!(
                "Invalid configuration {}: Failed to convert {:?} to u8",
                name,
                n
            )
        })
    })
}

fn parse_sat_amount(value: cln_plugin::options::Value, name: &str) -> Result<SatAmount> {
    let value = parse_opt_sat_amount(value, name)?;
    match value {
        Some(value) => Ok(value),
        None => Err(anyhow!("Missing configuration for '{}'", name)),
    }
}

fn parse_opt_sat_amount(
    value: cln_plugin::options::Value,
    name: &str,
) -> Result<Option<SatAmount>> {
    let value = parse_opt_str(value, name)?;
    match value {
        Some(value) => {
            let sat_value: u64 = value.parse::<u64>().context(format!(
                "Invalid config '{}': Could not convert {:?} to SatAmount",
                name, value
            ))?;
            return Ok(Some(SatAmount::new(sat_value)));
        }
        None => Ok(None),
    }
}

// TODO: We don't need the trait-bounds here.
// We should modify the cln-plugin crate to allow us to remove them
pub fn get_options<I, O>(plugin: &ConfiguredPlugin<PluginState, I, O>) -> Result<Lsps1Options>
where
    I: AsyncRead + Send + Unpin + 'static,
    O: AsyncWrite,
    O: Send,
    O: Unpin + 'static,
{
    let minimum_channel_confirmations: Value = plugin
        .option(LSPS1_MIN_CHANNEL_CONFIRMATIONS)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MIN_CHANNEL_CONFIRMATIONS
        ))?;
    let minimum_channel_confirmations: u8 = parse_num::<u8>(
        minimum_channel_confirmations,
        LSPS1_MIN_CHANNEL_CONFIRMATIONS,
    )?;

    let minimum_onchain_payment_confirmations: Value = plugin
        .option(LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS
        ))?;
    let minimum_onchain_payment_confirmations: u8 = parse_num::<u8>(
        minimum_onchain_payment_confirmations,
        LSPS1_MIN_ONCHAIN_PAYMENT_CONFIRMATIONS,
    )?;

    let supports_zero_channel_reserve: Value = plugin
        .option(LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE)
        .context(format!(
        "Option {} is not defined",
        LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE
    ))?;
    let supports_zero_channel_reserve: bool = parse_bool(
        supports_zero_channel_reserve,
        LSPS1_SUPPORTS_ZERO_CHANNEL_RESERVE,
    )?;

    let min_onchain_payment_size_sat: Value = plugin
        .option(LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT
        ))?;
    let min_onchain_payment_size_sat: Option<SatAmount> = parse_opt_sat_amount(
        min_onchain_payment_size_sat,
        LSPS1_MIN_ONCHAIN_PAYMENT_SIZE_SAT,
    )?;

    let max_channel_expiry_blocks: Value =
        plugin
            .option(LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS)
            .context(format!(
                "Option {} is not defined",
                LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS
            ))?;
    let max_channel_expiry_blocks: u32 =
        parse_num::<u32>(max_channel_expiry_blocks, LSPS1_MAX_CHANNEL_EXPIRY_BLOCKS)?;

    let min_initial_client_balance_sat: Value = plugin
        .option(LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT
        ))?;
    let min_initial_client_balance_sat: SatAmount = parse_sat_amount(
        min_initial_client_balance_sat,
        LSPS1_MIN_INITIAL_CLIENT_BALANCE_SAT,
    )?;

    let max_initial_client_balance_sat: Value = plugin
        .option(LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT
        ))?;
    let max_initial_client_balance_sat: SatAmount = parse_sat_amount(
        max_initial_client_balance_sat,
        LSPS1_MAX_INITIAL_CLIENT_BALANCE_SAT,
    )?;

    let min_initial_lsp_balance_sat: Value = plugin
        .option(LSPS1_MIN_INITIAL_LSP_BALANCE_SAT)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MIN_INITIAL_LSP_BALANCE_SAT
        ))?;
    let min_initial_lsp_balance_sat: SatAmount = parse_sat_amount(
        min_initial_lsp_balance_sat,
        LSPS1_MIN_INITIAL_LSP_BALANCE_SAT,
    )?;

    let max_initial_lsp_balance_sat: Value = plugin
        .option(LSPS1_MAX_INITIAL_LSP_BALANCE_SAT)
        .context(format!(
            "Option {} is not defined",
            LSPS1_MAX_INITIAL_LSP_BALANCE_SAT
        ))?;
    let max_initial_lsp_balance_sat: SatAmount = parse_sat_amount(
        max_initial_lsp_balance_sat,
        LSPS1_MAX_INITIAL_LSP_BALANCE_SAT,
    )?;

    let min_channel_balance_sat: Value =
        plugin
            .option(LSPS1_MIN_CHANNEL_BALANCE_SAT)
            .context(format!(
                "Option {} is not defined",
                LSPS1_MIN_CHANNEL_BALANCE_SAT
            ))?;
    let min_channel_balance_sat: SatAmount =
        parse_sat_amount(min_channel_balance_sat, LSPS1_MIN_CHANNEL_BALANCE_SAT)?;

    let max_channel_balance_sat: Value =
        plugin
            .option(LSPS1_MAX_CHANNEL_BALANCE_SAT)
            .context(format!(
                "Option {} is not defined",
                LSPS1_MAX_CHANNEL_BALANCE_SAT
            ))?;
    let max_channel_balance_sat: SatAmount =
        parse_sat_amount(max_channel_balance_sat, LSPS1_MAX_CHANNEL_BALANCE_SAT)?;

    Lsps1OptionsBuilder {
        min_channel_balance_sat: Some(min_channel_balance_sat),
        max_channel_balance_sat: Some(max_channel_balance_sat),
        min_initial_client_balance_sat: Some(min_initial_client_balance_sat),
        max_initial_client_balance_sat: Some(max_initial_client_balance_sat),
        min_initial_lsp_balance_sat: Some(min_initial_lsp_balance_sat),
        max_initial_lsp_balance_sat: Some(max_initial_lsp_balance_sat),
        minimum_channel_confirmations: Some(minimum_channel_confirmations),
        minimum_onchain_payment_confirmations: Some(minimum_onchain_payment_confirmations),
        supports_zero_channel_reserve: Some(supports_zero_channel_reserve),
        min_onchain_payment_size_sat: min_onchain_payment_size_sat,
        max_channel_expiry_blocks: Some(max_channel_expiry_blocks),
    }
    .build()
}

pub fn get_info<I, O>(plugin: &ConfiguredPlugin<PluginState, I, O>) -> Result<Lsps1InfoResponse>
where
    I: AsyncRead + Send + Unpin + 'static,
    O: AsyncWrite,
    O: Send,
    O: Unpin + 'static,
{
    let options = get_options(plugin)?;

    let website: Value = plugin
        .option(LSPS1_INFO_WEBSITE)
        .context(format!("Option {} is not defined", LSPS1_INFO_WEBSITE))?;
    let website: Option<String> = parse_opt_str(website, LSPS1_INFO_WEBSITE)?;

    Lsps1InfoResponseBuilder::default()
        .options(options)
        .website(website)
        .build()
}

pub fn get_state<I, O>(
    plugin: &ConfiguredPlugin<PluginState, I, O>,
) -> Result<Option<Lsps1InfoResponse>>
where
    I: AsyncRead + Send + Unpin + 'static,
    O: AsyncWrite,
    O: Send,
    O: Unpin + 'static,
{
    let lsps1_enabled = plugin
        .option(LSPS1_ENABLE)
        .unwrap()
        .as_bool()
        .unwrap_or(false);
    if lsps1_enabled {
        let info = get_info(&plugin)?;
        Ok(Some(info))
    } else {
        Ok(None)
    }
}
