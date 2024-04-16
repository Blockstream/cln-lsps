use anyhow::{Context, Result};

use cln_rpc::model::requests::{FeeratesRequest, FeeratesStyle};
use cln_rpc::model::responses::FeeratesPerkwEstimates;
use lsp_primitives::lsps0::common_schemas::SatAmount;

use crate::custom_msg::context::CustomMsgContext;
use crate::db::schema::Lsps1Order;
use crate::PluginState;

// TODO: Improve this API
// A practical fee calculator might want to have access to
// - ClnRpc: For Fee estimations + funds available
// - plugion.options or plugin.state: To see current configuration
// - ...
#[async_trait::async_trait]
pub trait FeeCalculator: Send {
    async fn calculate_fee(
        &self,
        context: &mut CustomMsgContext<PluginState>,
        order: Lsps1Order,
    ) -> Result<FeeCalculationResult>;
}

pub struct FeeCalculationResult {
    pub(crate) fee_total_sat: SatAmount,
    pub(crate) order_total_sat: SatAmount,
}

pub struct StandardFeeCalculator {
    pub fixed_msat: u64,
    pub weight_units: u64,
    pub sat_per_billion_sat_block: u64,
}

#[async_trait::async_trait]
impl FeeCalculator for StandardFeeCalculator {
    async fn calculate_fee(
        &self,
        context: &mut CustomMsgContext<PluginState>,
        order: Lsps1Order,
    ) -> Result<FeeCalculationResult> {
        // Compute the required onchain feerate
        // We use the lightning-rpc and confirms_within_blocks parameter
        let feerate_request = FeeratesRequest {
            style: FeeratesStyle::PERKW,
        };
        let feerate_response = context.cln_rpc.call_typed(&feerate_request).await?;
        let feerates = feerate_response
            .perkw
            .context("Failed to retreive feerates")?
            .estimates
            .context("Failed to retrieve feerates")?;

        let onchain_feerate_kwu = calculate_onchain_feerate(order.funding_confirms_within_blocks, &feerates)
            .context("Failed to compute approriate feerate")?
            as u64;

        // Compute the fee charged by the LSP
        self.calculate_lsp_fee(order, onchain_feerate_kwu)
    }
}

impl StandardFeeCalculator {
    fn calculate_lsp_fee(
        &self,
        order: Lsps1Order,
        onchain_feerate_sat_per_kwu: u64,
    ) -> Result<FeeCalculationResult> {
        let base_fee = self.fixed_msat;
        let client_balance_sat = order.client_balance_sat.sat_value() as u64;
        let channel_capacity =
            order.client_balance_sat.sat_value() + order.lsp_balance_sat.sat_value() as u64;
        let expiry_blocks = order.channel_expiry_blocks as u64;

        let fee_total_sat = base_fee + //fixed amount
            (onchain_feerate_sat_per_kwu*self.weight_units)/1000 + // Fee to compensate for onchain costs
            (channel_capacity * expiry_blocks)/1_000_000_000 * self.sat_per_billion_sat_block; // Fee for providing liquidity

        Ok(FeeCalculationResult {
            fee_total_sat: SatAmount::new(fee_total_sat),
            order_total_sat: SatAmount::new(fee_total_sat + client_balance_sat),
        })
    }
}

fn calculate_onchain_feerate(
    confirms_within_blocks: u8,
    feerates: &[FeeratesPerkwEstimates],
) -> Option<u32> {
    let max = feerates.iter().filter_map(|x| x.feerate).max();

    let result = feerates
        .iter()
        .filter(|x| x.blockcount.unwrap_or(0) as u64 <= confirms_within_blocks as u64)
        .filter_map(|x| x.feerate)
        .min();

    match result {
        Some(result) => Some(result),
        None => max,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cln_rpc::model::responses::FeeratesPerkwEstimates;

    #[test]
    fn test_calculate_fee_rate() {
        let feerates = vec![
            FeeratesPerkwEstimates {
                blockcount: Some(2),
                feerate: Some(10_000),
                smoothed_feerate: Some(10_000),
            },
            FeeratesPerkwEstimates {
                blockcount: Some(3),
                feerate: Some(9_000),
                smoothed_feerate: Some(9_000),
            },
            FeeratesPerkwEstimates {
                blockcount: Some(6),
                feerate: Some(5_000),
                smoothed_feerate: Some(5_000),
            },
            FeeratesPerkwEstimates {
                blockcount: Some(12),
                feerate: Some(2_000),
                smoothed_feerate: Some(2_000),
            },
        ];

        assert_eq!(calculate_onchain_feerate(1, &feerates), Some(10_000));
        assert_eq!(calculate_onchain_feerate(2, &feerates), Some(10_000));
        assert_eq!(calculate_onchain_feerate(3, &feerates), Some(9_000));
        assert_eq!(calculate_onchain_feerate(4, &feerates), Some(9_000));
        assert_eq!(calculate_onchain_feerate(5, &feerates), Some(9_000));
        assert_eq!(calculate_onchain_feerate(6, &feerates), Some(5_000));
    }
}
