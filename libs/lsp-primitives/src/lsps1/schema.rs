use crate::json_rpc::NoParams;
use crate::lsps0::common_schemas::{FeeRate, IsoDatetime, OnchainAddress, SatAmount};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Lsps1InfoRequest = NoParams;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1InfoResponse {
    pub supported_versions: Vec<u16>,
    pub website: Option<String>,
    pub options: Lsps1Options,
    // Prevents struct initialization. Use Lsps1InfoResponseBuilder instead
    #[serde(skip_serializing, default)]
    pub(crate) _private: (),
}

/// Options returned when calling lsps1.info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1Options {
    pub minimum_channel_confirmations: u8,
    pub minimum_onchain_payment_confirmations: Option<u8>,
    pub supports_zero_channel_reserve: bool,
    pub min_onchain_payment_size_sat: Option<SatAmount>,
    pub max_channel_expiry_blocks: u32,
    pub min_initial_client_balance_sat: SatAmount,
    pub max_initial_client_balance_sat: SatAmount,
    pub min_initial_lsp_balance_sat: SatAmount,
    pub max_initial_lsp_balance_sat: SatAmount,
    pub min_channel_balance_sat: SatAmount,
    pub max_channel_balance_sat: SatAmount,
    // Prevents struct initialization. Use Lsps1OptionsBuilder instead
    #[serde(skip_serializing, default)]
    pub(crate) _private: (),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1CreateOrderRequest {
    pub api_version: u16,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    pub refund_onchain_address: Option<OnchainAddress>,
    #[serde(rename = "announceChannel")]
    pub announce_channel: bool,
    // Prevents struct initialization. Use Lsps1OptionsBuilder instead
    #[serde(skip_serializing, default)]
    pub(crate) _private: (),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lsps1CreateOrderResponse {
    pub order_id: Uuid,
    pub api_version: u16,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: String,
    #[serde(rename = "announceChannel")]
    pub announce_channel: bool,
    pub created_at: IsoDatetime,
    pub expires_at: IsoDatetime,
    pub order_state: OrderState,

    pub payment: Payment,
    // Prevents struct initialization. Use Lsps1OptionsBuilder instead
    #[serde(skip_serializing, default)]
    pub(crate) _private: (),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderState {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    // Prevents struct initialization. Use OnchainPaymentBuilder instead
    #[serde(skip_serializing, default)]
    pub(crate) _private: (),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    pub state: PaymentState,
    pub fee_total_sat: SatAmount,
    pub order_total_sat: SatAmount,

    pub bolt11_invoice: String,
    pub onchain_address: Option<OnchainAddress>,
    pub required_onchain_block_confirmations: Option<u8>,

    pub minimum_fee_for_0conf: Option<FeeRate>,
    pub onchain_payment: Option<OnchainPayment>,

    // Prevents struct initialization. Use PaymentBuilder instead
    #[serde(skip_serializing, default)]
    pub(crate) _private: (),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lsps1GetOrderRequest {
    pub uuid: String,
    #[serde(skip_serializing, default)]
    _private: (),
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
            "minimum_channel_confirmations": 0,
            "minimum_onchain_payment_confirmations": 1,
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
             "minimum_channel_confirmations": 0,
             "minimum_onchain_payment_confirmations": 1,
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

        serde_json::from_value::<Lsps1InfoResponse>(json_data).unwrap();
    }

    #[test]
    fn lsps1_implements_serialize() {
        use core::str::FromStr;
        let onchain = OnchainAddress::from_str("32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf").unwrap();

        let request = Lsps1CreateOrderRequest {
            api_version: 1,
            lsp_balance_sat: SatAmount::new(100_000),
            client_balance_sat: SatAmount::new(1_000),
            confirms_within_blocks: 10,
            channel_expiry_blocks: 1000,
            token: None,
            refund_onchain_address: Some(onchain),
            announce_channel: false,
            _private: (),
        };

        let _ = serde_json::to_value(request).unwrap();
    }
}
