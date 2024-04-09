use anyhow::{Context, Result};
use lsp_primitives::lsps1::schema::Lsps1Options;
use lsp_primitives::methods::Lsps1GetInfoResponse;

use cln_plugin::ConfiguredPlugin;

use lsp_primitives::lsps0::common_schemas::SatAmount;
use lsp_primitives::lsps1::builders::{Lsps1InfoResponseBuilder, Lsps1OptionsBuilder};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::options;
use crate::state::PluginState;

fn create_sat_amount(amount: i64, name: &str) -> Result<SatAmount> {
    let amount: u64 = amount
        .try_into()
        .context(format!("{} should be a positive number", name))?;
    Ok(SatAmount::new(amount))
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
    let opt = options::lsps1_min_required_channel_confirmations();
    let min_required_channel_confirmations: u8 = plugin
        .option(&opt)
        .unwrap()
        .try_into()
        .context(format!("Option '{}' should be an u8", opt.name))?;

    let opt = options::lsps1_min_onchain_payment_confirmations();
    let min_onchain_payment_confirmations: Option<u8> = plugin
        .option(&opt)
        .unwrap()
        .map(|x| x.try_into())
        .transpose()
        .context("min_onchain_payment_confirmations does not fit into u8")?;

    let opt = options::lsps1_supports_zero_channel_reserve();
    let supports_zero_channel_reserve: bool = plugin.option(&opt).unwrap();

    let opt = options::lsps1_min_onchain_payment_size_sat();
    let min_onchain_payment_size_sat: Option<SatAmount> = plugin
        .option(&opt)
        .unwrap()
        .map(|x| create_sat_amount(x, opt.name))
        .transpose()?;

    let opt = options::lsps1_max_channel_expiry_blocks();
    let max_channel_expiry_blocks: u32 = plugin
        .option(&opt)
        .unwrap()
        .try_into()
        .context(format!("Option '{}' should fit into u32", opt.name))?;

    let opt = options::lsps1_min_initial_client_balance_sat();
    let min_initial_client_balance_sat: i64 = plugin
        .option(&opt)
        .unwrap()
        .context(format!("No value set for option {}", opt.name()))?;
    let min_initial_client_balance_sat: SatAmount =
        create_sat_amount(min_initial_client_balance_sat, opt.name)?;

    let opt = options::lsps1_max_initial_client_balance_sat();
    let max_initial_client_balance_sat: i64 = plugin
        .option(&opt)
        .unwrap()
        .context(format!("No value set for option {}", opt.name))?;
    let max_initial_client_balance_sat =
        create_sat_amount(max_initial_client_balance_sat, opt.name)?;

    let opt = options::lsps1_min_initial_lsp_balance_sat();
    let min_initial_lsp_balance_sat: i64 = plugin
        .option(&opt)
        .unwrap()
        .context(format!("No value set for option {}", opt.name))?;
    let min_initial_lsp_balance_sat: SatAmount =
        create_sat_amount(min_initial_lsp_balance_sat, opt.name)?;

    let opt = options::lsps1_max_initial_lsp_balance_sat();
    let max_initial_lsp_balance_sat: i64 = plugin
        .option(&opt)
        .unwrap()
        .context(format!("No value set for option {}", opt.name))?;
    let max_initial_lsp_balance_sat: SatAmount =
        create_sat_amount(max_initial_lsp_balance_sat, opt.name)?;

    let opt = options::lsps1_min_channel_balance_sat();
    let min_channel_balance_sat: i64 = plugin
        .option(&opt)
        .unwrap()
        .context(format!("No value set for option {}", opt.name))?;
    let min_channel_balance_sat: SatAmount = create_sat_amount(min_channel_balance_sat, opt.name)?;

    let opt = options::lsps1_max_channel_balance_sat();
    let max_channel_balance_sat: i64 = plugin
        .option(&opt)
        .unwrap()
        .context(format!("No value set for option {}", opt.name))?;
    let max_channel_balance_sat: SatAmount = create_sat_amount(max_channel_balance_sat, opt.name)?;

    let opt = options::lsps1_min_funding_confirms_within_blocks();
    let min_funding_confirms_within_blocks: u8 = plugin
        .option(&opt)
        .unwrap()
        .try_into()
        .context(format!("{} should fit into u8", opt.name))?;

    Lsps1OptionsBuilder {
        min_funding_confirms_within_blocks: Some(min_funding_confirms_within_blocks),
        min_channel_balance_sat: Some(min_channel_balance_sat),
        max_channel_balance_sat: Some(max_channel_balance_sat),
        min_initial_client_balance_sat: Some(min_initial_client_balance_sat),
        max_initial_client_balance_sat: Some(max_initial_client_balance_sat),
        min_initial_lsp_balance_sat: Some(min_initial_lsp_balance_sat),
        max_initial_lsp_balance_sat: Some(max_initial_lsp_balance_sat),
        min_required_channel_confirmations: Some(min_required_channel_confirmations),
        supports_zero_channel_reserve: Some(supports_zero_channel_reserve),
        max_channel_expiry_blocks: Some(max_channel_expiry_blocks),
        min_onchain_payment_confirmations,
        min_onchain_payment_size_sat,
    }
    .build()
}

pub fn get_info<I, O>(plugin: &ConfiguredPlugin<PluginState, I, O>) -> Result<Lsps1GetInfoResponse>
where
    I: AsyncRead + Send + Unpin + 'static,
    O: AsyncWrite,
    O: Send,
    O: Unpin + 'static,
{
    let options = get_options(&plugin)?;

    Lsps1InfoResponseBuilder::default().options(options).build()
}

pub fn get_state<I, O>(
    plugin: &ConfiguredPlugin<PluginState, I, O>,
) -> Result<Option<Lsps1GetInfoResponse>>
where
    I: AsyncRead + Send + Unpin + 'static,
    O: AsyncWrite,
    O: Send,
    O: Unpin + 'static,
{
    let lsps1_enabled = plugin.option(&options::lsps1_enable()).unwrap();

    if lsps1_enabled {
        let info = get_info(&plugin)?;
        Ok(Some(info))
    } else {
        Ok(None)
    }
}
