use anyhow::{anyhow, Context, Result};

use lsp_primitives::lsps0::common_schemas::SatAmount;
use lsp_primitives::lsps1::builders::{Lsps1InfoResponseBuilder, Lsps1OptionsBuilder};
use lsp_primitives::lsps1::schema::Lsps1InfoResponse;

use cln_plugin::options::{ConfigOption, Value};

// TODO: Split the create options
pub fn info_response(config_options: Vec<ConfigOption>) -> Result<Lsps1InfoResponse> {
    let mut website = Option::<String>::None;
    let mut lsps1_minimum_channel_confirmations = Option::<u8>::None;
    let mut lsps1_minimum_onchain_payment_confirmations = Option::<u8>::None;
    let mut lsps1_supports_zero_channel_reserve = Option::<bool>::None;
    let mut lsps1_max_channel_expiry_blocks = Option::<u32>::None;
    let mut lsps1_min_onchain_payment_size_sat = Option::<SatAmount>::None;
    let mut lsps1_min_capacity = Option::<SatAmount>::None;
    let mut lsps1_max_capacity = Option::<SatAmount>::None;

    for opt in config_options {
        log::debug!("Reading LSPS1 option: {}={:?}", opt.name(), opt.value());
        match opt.name() {
            "lsps1_info_website" => {
                let value = opt.value().as_str().map(|x| String::from(x));
                website = value;
            }
            "lsps1_minimum_channel_confirmations" => {
                let value = opt.value().as_i64().ok_or(anyhow!(
                    "Invalid or missing configuration for `lsps1_minimum_channel_confirmations`"
                ))?;
                let u8_value = u8::try_from(value).context(
                    "`lsps1_minimum_channel_configuration` should fit in 8-bit unsigned integer",
                )?;
                lsps1_minimum_channel_confirmations = Some(u8_value);
            }
            "lsps1_minimum_onchain_payment_confirmations" => {
                if is_some(opt.value()) {
                    let value = opt.value().as_i64();
                    let u8_value = value.map(|x| u8::try_from(x)).transpose().context("'lsps1_minimum_onchain_payment_confirmations should fit in 8-bit unsigned integer")?;
                    lsps1_minimum_onchain_payment_confirmations = u8_value;
                }
            }
            "lsps1_supports_zero_channel_reserve" => {
                let value = opt.value().as_bool().context(
                    "Invalid or missing configuration for `lsps1_supports_zero_channel_reserve`",
                )?;
                lsps1_supports_zero_channel_reserve = Some(value);
            }
            "lsps1_max_channel_expiry_blocks" => {
                let value = opt.value().as_i64().context(
                    "Invalid or missing configuration for `lsps1_max_channel_expiry_blocks`",
                )?;
                let value_u32 = u32::try_from(value).context(
                    "'lsps1_max_channel_expiry_blocks' should fit in 32-bit unsigned integer",
                )?;
                lsps1_max_channel_expiry_blocks = Some(value_u32);
            }
            "lsps1_min_onchain_payment_size_sat" => {
                // Might be `null` if LSP does not support onchain paymnets
                if is_some(opt.value()) {
                    log::info!("{}={:?}", opt.name(), opt.value());
                    lsps1_min_onchain_payment_size_sat = Some(parse_option_to_sat(&opt)?);
                }
            }
            "lsps1_min_capacity" => {
                lsps1_min_capacity = Some(parse_option_to_sat(&opt)?);
            }
            "lsps1_max_capacity" => {
                lsps1_max_capacity = Some(parse_option_to_sat(&opt)?);
            }
            _ => {
                // The plugin defines additional config options
                // We are free to ignore them
            }
        }
    }

    let options = Lsps1OptionsBuilder::new()
        .minimum_channel_confirmations(
            lsps1_minimum_channel_confirmations
                .context("No config specified for `lsps1_minimum_channel_confirmations`")?,
        )
        .supports_zero_channel_reserve(
            lsps1_supports_zero_channel_reserve
                .context("No config specified for `lsps1_supports_zero_channel_reserve`")?,
        )
        .max_channel_expiry_blocks(
            lsps1_max_channel_expiry_blocks
                .context("No config specified for `lsps1_max_channel_expiry_blocks`")?,
        )
        .min_onchain_payment_size_sat(lsps1_min_onchain_payment_size_sat)
        .minimum_onchain_payment_confirmations(lsps1_minimum_onchain_payment_confirmations)
        .min_initial_lsp_balance_sat(
            lsps1_min_capacity
                .clone()
                .context("No config specified for `lsps1_min_capacity`")?,
        )
        .max_initial_lsp_balance_sat(
            lsps1_max_capacity
                .clone()
                .context("No config specified for `lsps1_max_capacity`")?,
        )
        .min_channel_balance_sat(
            lsps1_min_capacity
                .clone()
                .context("No config specified for `lsps1_min_capacity`")?,
        )
        .max_channel_balance_sat(
            lsps1_max_capacity
                .clone()
                .context("No config specified for `lsps1_max_capacity`")?,
        )
        .min_initial_client_balance_sat(SatAmount::new(0))
        .max_initial_client_balance_sat(SatAmount::new(0))
        .build()?;

    Lsps1InfoResponseBuilder::new()
        .options(options)
        .website(website)
        .build()
}

// TODO: Get rid of this function once an equivalent is backported
// to cln_plugin
//
// https://github.com/ElementsProject/lightning/pull/6894
fn is_some(value: Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Integer(_) => true,
        Value::Boolean(_) => true,
        Value::OptString => false,
        Value::OptInteger => false,
        Value::OptBoolean => false,
    }
}

fn parse_option_to_sat(option: &ConfigOption) -> Result<SatAmount> {
    let value = option.value();
    let str_value = value.as_str().context(format!(
        "Invalid or missing configuration for '{}'. Use a String to specify the SatAmount",
        option.name()
    ))?;
    let str_value = str_value.replace("_", "");
    let value_u64 = str_value.parse::<u64>().context(format!(
        "Invalid value for '{}'. The number should be a positive amount of sats",
        option.name()
    ))?;
    Ok(SatAmount::new(value_u64))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_option_to_sat_amount() {
        let value_pairs = vec![
            (100_000, Value::String("100_000".to_string())),
            (0, Value::String("000".to_string())),
        ];

        for pair in value_pairs {
            let option = ConfigOption::new("name", pair.1, "Some description");
            assert_eq!(
                SatAmount::new(pair.0),
                parse_option_to_sat(&option).unwrap()
            )
        }
    }
}
