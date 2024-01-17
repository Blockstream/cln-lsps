use anyhow::Result;

use lsp_primitives::lsps0::common_schemas::SatAmount;

use crate::custom_msg::context::CustomMsgContext;
use crate::db::schema::Lsps1Order;
use crate::PluginState;

// TODO: Improve this API
// A practical fee calculator might want to have access to
// - ClnRpc: For Fee estimations + funds available
// - plugion.options or plugin.state: To see current configuration
// - ...
pub trait FeeCalculator: Send {
    fn calculate_fee(
        &self,
        context: &mut CustomMsgContext<PluginState>,
        order: Lsps1Order,
    ) -> Result<FeeCalculationResult>;
}

pub struct FixedFeeCalculator {
    fixed_fee: SatAmount,
}

pub struct FeeCalculationResult {
    pub(crate) fee_total_sat: SatAmount,
    pub(crate) order_total_sat: SatAmount,
}

impl FixedFeeCalculator {
    pub fn new(fixed_fee: SatAmount) -> Self {
        Self { fixed_fee }
    }
}

impl FeeCalculator for FixedFeeCalculator {
    // TODO: Change this API and ensure we pass some context
    // We should at least be able to query
    fn calculate_fee(
        &self,
        _context: &mut CustomMsgContext<PluginState>,
        order: Lsps1Order,
    ) -> Result<FeeCalculationResult> {
        let base_fee: u64 = self.fixed_fee.sat_value();

        let fee_total_sat = base_fee;
        let order_total_sat = fee_total_sat + order.client_balance_sat.sat_value();

        Ok(FeeCalculationResult {
            fee_total_sat: SatAmount::new(fee_total_sat),
            order_total_sat: SatAmount::new(order_total_sat),
        })
    }
}
