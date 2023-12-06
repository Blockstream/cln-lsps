use anyhow::{Context, Result};
use lsp_primitives::lsps0::common_schemas::{
    FeeRate, IsoDatetime, NetworkChecked, PublicKey, SatAmount,
};
use lsp_primitives::lsps1::schema::{Lsps1CreateOrderRequest, OrderState};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Lsps1Order {
    pub(crate) uuid: Uuid,
    pub(crate) client_node_id: PublicKey,
    pub(crate) lsp_balance_sat: SatAmount,
    pub(crate) client_balance_sat: SatAmount,
    pub(crate) confirms_within_blocks: u8,
    pub(crate) channel_expiry_blocks: u32,
    pub(crate) token: Option<String>,
    pub(crate) refund_onchain_address: Option<String>,
    pub(crate) announce_channel: bool,
    pub(crate) created_at: IsoDatetime,
    pub(crate) expires_at: IsoDatetime,
    pub(crate) order_state: OrderState,
}

#[derive(Default)]
pub struct Lsps1OrderBuilder {
    pub(crate) uuid: Option<Uuid>,
    pub(crate) client_node_id: Option<PublicKey>,
    pub(crate) lsp_balance_sat: Option<SatAmount>,
    pub(crate) client_balance_sat: Option<SatAmount>,
    pub(crate) confirms_within_blocks: Option<u8>,
    pub(crate) channel_expiry_blocks: Option<u32>,
    pub(crate) token: Option<String>,
    pub(crate) refund_onchain_address: Option<String>,
    pub(crate) announce_channel: Option<bool>,
    pub(crate) created_at: Option<IsoDatetime>,
    pub(crate) expires_at: Option<IsoDatetime>,
    pub(crate) order_state: Option<OrderState>,
}

#[derive(Debug, Clone)]
pub struct Lsps1PaymentDetails {
    pub(crate) fee_total_sat: SatAmount,
    pub(crate) order_total_sat: SatAmount,
    pub(crate) bolt11_invoice: String,
    pub(crate) onchain_address: Option<String>,
    pub(crate) onchain_block_confirmations_required: Option<u8>,
    pub(crate) minimum_fee_for_0conf: Option<FeeRate>,
}

#[derive(Default)]
pub struct Lsps1PaymentDetailsBuilder {
    pub(crate) fee_total_sat: Option<SatAmount>,
    pub(crate) order_total_sat: Option<SatAmount>,
    pub(crate) bolt11_invoice: Option<String>,
    pub(crate) onchain_address: Option<String>,
    pub(crate) onchain_block_confirmations_required: Option<u8>,
    pub(crate) minimum_fee_for_0conf: Option<FeeRate>,
}

impl Lsps1OrderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    pub fn client_node_id(mut self, client_node_id: PublicKey) -> Self {
        self.client_node_id = Some(client_node_id);
        self
    }

    pub fn client_balance_sat(mut self, client_balance_sat: SatAmount) -> Self {
        self.client_balance_sat = Some(client_balance_sat);
        self
    }

    pub fn lsp_balance_sat(mut self, lsp_balance_sat: SatAmount) -> Self {
        self.lsp_balance_sat = Some(lsp_balance_sat);
        self
    }

    pub fn confirms_within_blocks(mut self, confirms_within_blocks: u8) -> Self {
        self.confirms_within_blocks = Some(confirms_within_blocks);
        self
    }

    pub fn channel_expiry_blocks(mut self, channel_expiry_blocks: Option<u32>) -> Self {
        self.channel_expiry_blocks = channel_expiry_blocks;
        self
    }

    pub fn token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    pub fn refund_onchain_address(mut self, refund_onchain_address: Option<String>) -> Self {
        self.refund_onchain_address = refund_onchain_address;
        self
    }

    pub fn announce_channel(mut self, announce_channel: bool) -> Self {
        self.announce_channel = Some(announce_channel);
        self
    }

    pub fn created_at(mut self, created_at: IsoDatetime) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn expires_at(mut self, expires_at: IsoDatetime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn order_request(
        mut self,
        order_request: &Lsps1CreateOrderRequest<NetworkChecked>,
    ) -> Self {
        self.client_balance_sat = Some(order_request.client_balance_sat);
        self.lsp_balance_sat = Some(order_request.lsp_balance_sat);
        self.confirms_within_blocks = Some(order_request.confirms_within_blocks);
        self.channel_expiry_blocks = Some(order_request.channel_expiry_blocks);
        self.token = order_request.token.clone();
        self.refund_onchain_address = order_request
            .refund_onchain_address
            .clone()
            .map(|address| address.to_string());
        self.announce_channel = Some(order_request.announce_channel);
        self.lsp_balance_sat = Some(order_request.lsp_balance_sat);
        self
    }

    pub fn order_state(mut self, order_state: OrderState) -> Self {
        self.order_state = Some(order_state);
        self
    }

    pub fn build(self) -> Result<Lsps1Order> {
        // Fields that must have a value
        let client_balance_sat = self
            .client_balance_sat
            .context("Missing field 'client_balance_dat'")?;
        let lsp_balance_sat = self
            .lsp_balance_sat
            .context("Missing field 'lsp_balance_sat'")?;
        let client_node_id = self
            .client_node_id
            .context("Missing field 'client_node_id'")?;
        let channel_expiry_blocks = self
            .channel_expiry_blocks
            .context("Missing field 'channel_expiry_blocks'")?;
        let confirms_within_blocks = self
            .confirms_within_blocks
            .context("Missing field 'confirms_within_blocks'")?;
        let created_at = self.created_at.context("Missing field 'created_at'")?;
        let expires_at = self.expires_at.context("Missing field 'expires_at'")?;
        let announce_channel = self
            .announce_channel
            .context("Missing field 'announce_channel'")?;

        // Fields with sensible defaults
        let uuid: Uuid = self.uuid.unwrap_or_else(|| Uuid::new_v4());
        let order_state = self.order_state.unwrap_or(OrderState::Created);

        Ok(Lsps1Order {
            uuid,
            client_balance_sat,
            client_node_id,
            channel_expiry_blocks,
            lsp_balance_sat,
            confirms_within_blocks,
            created_at,
            expires_at,
            token: self.token,
            refund_onchain_address: self.refund_onchain_address,
            announce_channel,
            order_state,
        })
    }
}

impl Lsps1PaymentDetailsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fee_total_sat(mut self, fee_total_sat: SatAmount) -> Self {
        self.fee_total_sat = Some(fee_total_sat);
        self
    }

    pub fn order_total_sat(mut self, order_total_sat: SatAmount) -> Self {
        self.order_total_sat = Some(order_total_sat);
        self
    }

    pub fn bolt11_invoice(mut self, bolt11_invoice: String) -> Self {
        self.bolt11_invoice = Some(bolt11_invoice);
        self
    }

    pub fn onchain_address(mut self, onchain_address: Option<String>) -> Self {
        self.onchain_address = onchain_address;
        self
    }

    pub fn onchain_block_confirmations_required(
        mut self,
        onchain_block_confirmations_required: Option<u8>,
    ) -> Self {
        self.onchain_block_confirmations_required = onchain_block_confirmations_required;
        self
    }

    pub fn minimum_fee_for_0conf(mut self, minimum_fee_for_0conf: Option<FeeRate>) -> Self {
        self.minimum_fee_for_0conf = minimum_fee_for_0conf;
        self
    }

    pub fn build(self) -> Result<Lsps1PaymentDetails> {
        Ok(Lsps1PaymentDetails {
            fee_total_sat: self
                .fee_total_sat
                .context("Missing field 'fee_total_sat'")?,
            order_total_sat: self
                .order_total_sat
                .context("Missing field 'order_total_sat'")?,
            bolt11_invoice: self
                .bolt11_invoice
                .context("Missing field 'bolt11_invoice'")?,
            onchain_address: self.onchain_address,
            onchain_block_confirmations_required: self.onchain_block_confirmations_required,
            minimum_fee_for_0conf: self.minimum_fee_for_0conf,
        })
    }
}
