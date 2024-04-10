use crate::json_rpc::NoParams;
use crate::lsps0::common_schemas::{FeeRate, IsoDatetime, OnchainAddress, Outpoint, SatAmount};
use crate::lsps0::parameter_validation::ExpectedFields;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Lsps1InfoRequest = NoParams;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1GetInfoResponse {
    pub website: Option<String>,
    pub options: Lsps1Options,
    // Prevents struct initialization. Use Lsps1InfoResponseBuilder instead
}

/// Options returned when calling lsps1.info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1Options {
    pub min_required_channel_confirmations: u8,
    pub min_funding_confirms_within_blocks: u8,
    pub min_onchain_payment_confirmations: Option<u8>,
    pub supports_zero_channel_reserve: bool,
    pub min_onchain_payment_size_sat: Option<SatAmount>,
    pub max_channel_expiry_blocks: u32,
    pub min_initial_client_balance_sat: SatAmount,
    pub max_initial_client_balance_sat: SatAmount,
    pub min_initial_lsp_balance_sat: SatAmount,
    pub max_initial_lsp_balance_sat: SatAmount,
    pub min_channel_balance_sat: SatAmount,
    pub max_channel_balance_sat: SatAmount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1CreateOrderRequest {
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    pub refund_onchain_address: Option<OnchainAddress>,
    pub announce_channel: bool,
}

impl ExpectedFields for Lsps1CreateOrderRequest {
    fn expected_fields() -> Vec<String> {
        vec![
            "lsp_balance_sat".to_string(),
            "client_balance_sat".to_string(),
            "confirms_within_blocks".to_string(),
            "channel_expiry_blocks".to_string(),
            "token".to_string(),
            "refund_onchain_address".to_string(),
            "announce_channel".to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1CreateOrderResponse {
    pub order_id: Uuid,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: String,
    pub announce_channel: bool,
    pub created_at: IsoDatetime,
    pub expires_at: IsoDatetime,
    pub order_state: OrderState,

    pub payment: Payment,
    pub channel: Option<Channel>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum OrderState {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum PaymentState {
    #[serde(rename = "EXPECT_PAYMENT")]
    ExpectPayment,
    #[serde(rename = "HOLD")]
    Hold,
    #[serde(rename = "PAID")]
    Paid,
    #[serde(rename = "REFUNDED")]
    Refunded,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnchainPayment {
    pub outpoint: String,
    pub sat: SatAmount,
    pub confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    pub state: PaymentState,
    pub fee_total_sat: SatAmount,
    pub order_total_sat: SatAmount,

    pub bolt11_invoice: String,
    pub onchain_address: Option<OnchainAddress>,
    pub min_onchain_payment_confirmations: Option<u8>,

    pub min_fee_for_0conf: Option<FeeRate>,
    pub onchain_payment: Option<OnchainPayment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub funded_at: IsoDatetime,
    pub funding_outpoint: Outpoint,
    pub expires_at: IsoDatetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lsps1GetOrderRequest {
    pub order_id: String,
}

impl ExpectedFields for Lsps1GetOrderRequest {
    fn expected_fields() -> Vec<String> {
        vec!["order_id".to_string()]
    }
}

pub type Lsps1GetOrderResponse = Lsps1CreateOrderResponse;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn deserialize_lsps1_options() {
        // Example is copie pasted from the spec
        let json_data = serde_json::json!(
        {
            "min_required_channel_confirmations": 0,
            "min_onchain_payment_confirmations": 1,
            "min_funding_confirms_within_blocks": 10,
            "supports_zero_channel_reserve": true,
            "min_onchain_payment_size_sat": null,
            "max_channel_expiry_blocks": 20160,
            "min_initial_client_balance_sat": "20000",
            "max_initial_client_balance_sat": "100000000",
            "min_initial_lsp_balance_sat": "0",
            "max_initial_lsp_balance_sat": "100000000",
            "min_channel_balance_sat": "50000",
            "max_channel_balance_sat": "100000000"
        }

        );
        serde_json::from_value::<Lsps1Options>(json_data).unwrap();
    }

    #[test]
    fn deserialize_lsps1_info() {
        // Example below is copied from the spec
        let json_data = serde_json::json!(
        {
         "supported_versions": [1],
         "website": "http://example.com/contact",
         "options": {
             "min_required_channel_confirmations": 0,
             "min_funding_confirms_within_blocks": 12,
             "min_onchain_payment_confirmations": 1,
             "supports_zero_channel_reserve": true,
             "min_onchain_payment_size_sat": null,
             "max_channel_expiry_blocks": 20160,
             "min_initial_client_balance_sat": "20000",
             "max_initial_client_balance_sat": "100000000",
             "min_initial_lsp_balance_sat": "0",
             "max_initial_lsp_balance_sat": "100000000",
             "min_channel_balance_sat": "50000",
             "max_channel_balance_sat": "100000000"
         }
        }
        );

        serde_json::from_value::<Lsps1GetInfoResponse>(json_data).unwrap();
    }

    #[test]
    fn lsps1_implements_serialize() {
        use core::str::FromStr;
        let onchain = OnchainAddress::from_str("32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf").unwrap();

        let request = Lsps1CreateOrderRequest {
            lsp_balance_sat: SatAmount::new(100_000),
            client_balance_sat: SatAmount::new(1_000),
            confirms_within_blocks: 10,
            channel_expiry_blocks: 1000,
            token: None,
            refund_onchain_address: Some(onchain),
            announce_channel: false,
        };

        let _ = serde_json::to_value(request).unwrap();
    }
}
