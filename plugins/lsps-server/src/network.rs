use anyhow::{anyhow, Result};
use lsp_primitives::lsps0::common_schemas::Network;

pub fn parse_network(network: &str) -> Result<Network> {
    let result = match network {
        "bitcoin" => Network::Bitcoin,
        "regtest" => Network::Regtest,
        "testnet" => Network::Testnet,
        "signet" => Network::Signet,
        _ => return Err(anyhow!("Unknown network '{:?}'", network)),
    };

    return Ok(result);
}
