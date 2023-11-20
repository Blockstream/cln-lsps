use crate::json_rpc::NoParams;
use crate::lsps0::common_schemas::{IsoDatetime, SatAmount};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type OnchainFeeRate = u64;

pub type Lsps1InfoRequest = NoParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1InfoResponse {
    pub supported_versions: Vec<u16>,
    pub website: Option<String>,
    pub options: Lsps1Options,
    // Prevents struct initialization. Use Lsps1InfoResponseBuilder instead
    pub(crate)_private : ()
}

/// Options returned when calling lsps1.info
#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1Options {
    pub minimum_channel_confirmations: u8,
    pub minimum_onchain_payment_confirmations: Option<u8>,
    pub supports_zero_channel_reserve: bool,
    pub min_onchain_payment_size_sat: Option<u64>,
    pub max_channel_expiry_blocks: u32,
    pub min_initial_client_balance_sat: SatAmount,
    pub max_initial_client_balance_sat: SatAmount,
    pub min_initial_lsp_balance_sat: SatAmount,
    pub max_initial_lsp_balance_sat : SatAmount,
    pub min_channel_balance_sat: SatAmount,
    pub max_channel_balance_sat: SatAmount,
    // Prevents struct initialization. Use Lsps1OptionsBuilder instead
    pub(crate)_private : ()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1GetOrderRequest {
    pub api_version: u16,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    pub refund_onchain_address: Option<String>,
    #[serde(rename = "announceChannel")]
    pub announce_channel: bool,
    // Prevents struct initialization. Use Lsps1OptionsBuilder instead
    pub(crate)_private : ()

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1GetOrderResponse {
    pub order_id: Uuid,
    pub api_version: u16,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u8,
    pub token: String,
    #[serde(rename = "announceChannel")]
    pub announce_channel: bool,
    pub created_at: IsoDatetime,
    pub expires_at: IsoDatetime,
    pub order_state: OrderState,
    pub payment: Payment,
    // Prevents struct initialization. Use Lsps1OptionsBuilder instead
    pub(crate) _private : ()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderState {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentState {
    #[serde(rename = "EXPECT_PAYMENT")]
    ExpectPayment,
    #[serde(rename = "HOLD")]
    Hold,
    #[serde(rename = "STATE")]
    State,
    #[serde(rename = "REFUNDED")]
    Refunded,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnchainPayment {
    pub outpoint: String,
    pub sat: SatAmount,
    pub confirmed: bool,
    // Prevents struct initialization. Use OnchainPaymentBuilder instead
    pub(crate) _private : ()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payment {
    pub state: PaymentState,
    pub fee_total_sat: SatAmount,
    pub order_total_sat: SatAmount,

    pub bolt11_invoice: String,
    pub onchain_address: Option<String>,
    pub required_onchain_block_confirmations: Option<u8>,

    pub minimum_fee_for_0conf: Option<OnchainFeeRate>,
    pub onchain_payment: Option<OnchainPayment>,

    // Prevents struct initialization. Use PaymentBuilder instead
    pub(crate) _private : ()
}

