use anyhow::{anyhow, Context, Result};
use uuid::Uuid;

use crate::lsps0::schema::{FeeRate, IsoDatetime, OnchainAddress, SatAmount};
use crate::lsps1::schema::{
    Channel, Lsps1CreateOrderRequest, Lsps1CreateOrderResponse, Lsps1GetInfoResponse,
    Lsps1GetOrderRequest, Lsps1InfoRequest, Lsps1Options, OnchainPayment, OrderState, Payment,
    PaymentState,
};

#[derive(Default, Debug)]
pub struct LspsInfoRequestBuilder;

impl LspsInfoRequestBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build() -> Lsps1InfoRequest {
        Lsps1InfoRequest {}
    }
}

#[derive(Default, Debug)]
pub struct Lsps1InfoResponseBuilder {
    options: Option<Lsps1Options>,
}

impl Lsps1InfoResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: Lsps1Options) -> Self {
        self.options = Some(options);
        self
    }

    pub fn build(self) -> Result<Lsps1GetInfoResponse> {
        let options = self
            .options
            .context("Missing field 'options' in Lsps1InfoResponseBuilder")?;

        let result = Lsps1GetInfoResponse { options };
        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct Lsps1OptionsBuilder {
    pub min_required_channel_confirmations: Option<u8>,
    pub min_funding_confirms_within_blocks: Option<u8>,
    pub min_onchain_payment_confirmations: Option<u8>,
    pub supports_zero_channel_reserve: Option<bool>,
    pub min_onchain_payment_size_sat: Option<SatAmount>,
    pub max_channel_expiry_blocks: Option<u32>,
    pub min_initial_client_balance_sat: Option<SatAmount>,
    pub max_initial_client_balance_sat: Option<SatAmount>,
    pub min_initial_lsp_balance_sat: Option<SatAmount>,
    pub max_initial_lsp_balance_sat: Option<SatAmount>,
    pub min_channel_balance_sat: Option<SatAmount>,
    pub max_channel_balance_sat: Option<SatAmount>,
}

impl Lsps1OptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_required_channel_confirmations(
        mut self,
        min_required_channel_confirmations: u8,
    ) -> Self {
        self.min_required_channel_confirmations = Some(min_required_channel_confirmations);
        self
    }

    pub fn min_funding_confirms_within_blocks(
        mut self,
        min_funding_confirms_within_blocks: u8,
    ) -> Self {
        self.min_funding_confirms_within_blocks = Some(min_funding_confirms_within_blocks);
        self
    }

    pub fn min_onchain_payment_confirmations(
        mut self,
        min_onchain_payment_confirmations: Option<u8>,
    ) -> Self {
        self.min_onchain_payment_confirmations = min_onchain_payment_confirmations;
        self
    }

    pub fn supports_zero_channel_reserve(mut self, supports_zero_channel_reserve: bool) -> Self {
        self.supports_zero_channel_reserve = Some(supports_zero_channel_reserve);
        self
    }

    pub fn min_onchain_payment_size_sat(
        mut self,
        min_onchain_payment_size_sat: Option<SatAmount>,
    ) -> Self {
        self.min_onchain_payment_size_sat = min_onchain_payment_size_sat;
        self
    }

    pub fn max_channel_expiry_blocks(mut self, max_channel_expiry_blocks: u32) -> Self {
        self.max_channel_expiry_blocks = Some(max_channel_expiry_blocks);
        self
    }

    pub fn min_initial_client_balance_sat(
        mut self,
        min_initial_client_balance_sat: SatAmount,
    ) -> Self {
        self.min_initial_client_balance_sat = Some(min_initial_client_balance_sat);
        self
    }

    pub fn max_initial_client_balance_sat(
        mut self,
        max_initial_client_balance_sat: SatAmount,
    ) -> Self {
        self.max_initial_client_balance_sat = Some(max_initial_client_balance_sat);
        self
    }

    pub fn min_initial_lsp_balance_sat(mut self, min_initial_lsp_balance_sat: SatAmount) -> Self {
        self.min_initial_lsp_balance_sat = Some(min_initial_lsp_balance_sat);
        self
    }

    pub fn max_initial_lsp_balance_sat(mut self, max_initial_lsp_balance_sat: SatAmount) -> Self {
        self.max_initial_lsp_balance_sat = Some(max_initial_lsp_balance_sat);
        self
    }

    pub fn min_channel_balance_sat(mut self, min_channel_balance_sat: SatAmount) -> Self {
        self.min_channel_balance_sat = Some(min_channel_balance_sat);
        self
    }

    pub fn max_channel_balance_sat(mut self, max_channel_balance_sat: SatAmount) -> Self {
        self.max_channel_balance_sat = Some(max_channel_balance_sat);
        self
    }

    pub fn build(self) -> Result<Lsps1Options> {
        let min_required_channel_confirmations = self.min_required_channel_confirmations.context(
            "No value specified for 'min_required_channel_confirmations' in Lsps1OptionBuilder",
        )?;
        let min_funding_confirms_within_blocks = self.min_funding_confirms_within_blocks.context(
            "No value specified for 'min_funding_confirms_within_blocks' in Lsps1OptionBuilder",
        )?;
        let supports_zero_channel_reserve = self.supports_zero_channel_reserve.context(
            "No value specified for 'supports_zero_channel_reserve' in Lsps1OptionsBuilder",
        )?;
        let max_channel_expiry_blocks = self
            .max_channel_expiry_blocks
            .context("No value specified for 'max_channel_expiry_blocks' in Lsps1OptionsBuilder")?;
        let min_initial_lsp_balance_sat = self.min_initial_lsp_balance_sat.context(
            "No value specified for 'min_initial_lsp_balance_sat' in Lsps1OptionsBuilder",
        )?;
        let max_initial_lsp_balance_sat = self.max_initial_lsp_balance_sat.context(
            "No value specified for 'max_initial_lsp_balance_sat' in Lsps1OptionsBuilder",
        )?;

        let min_initial_client_balance_sat = self.min_initial_client_balance_sat.context(
            "No value specified for 'min_initial_client_balance_sat' in Lsps1OptionsBuilder",
        )?;
        let max_initial_client_balance_sat = self.max_initial_client_balance_sat.context(
            "No value specified for 'max_initial_client_balance_sat' in Lsps1OptionsBuilder",
        )?;
        let min_channel_balance_sat = self
            .min_channel_balance_sat
            .context("No value specified for 'min_channel_balance_sat' in Lsps1OptionsBuilder")?;
        let max_channel_balance_sat = self
            .max_channel_balance_sat
            .context("No value specified for 'max_channel_balance_sat' in Lsps1OptionsBuilder")?;

        // Maybe NULL if the LSP doesn't support on chain payments
        let min_onchain_payment_size_sat = self.min_onchain_payment_size_sat;
        let min_onchain_payment_confirmations = self.min_onchain_payment_confirmations;

        if min_channel_balance_sat > max_channel_balance_sat {
            return Err(anyhow!("min_channel_balance_sat ({}) should be less than or equal to max_channel_balance_sat ({})", min_channel_balance_sat, max_channel_balance_sat));
        }
        if min_initial_client_balance_sat > max_initial_client_balance_sat {
            return Err(anyhow!("min_initial_client_balance_sat ({}) should be less than or equal to max_initial_client_balance_sat ({})", min_initial_client_balance_sat, max_initial_client_balance_sat));
        }
        if min_initial_lsp_balance_sat > max_initial_lsp_balance_sat {
            return Err(anyhow!("min_initial_lsp_balance_sat ({}) should be less than or equal to max_initial_lsp_balance_sat ({})", min_initial_lsp_balance_sat, max_initial_lsp_balance_sat));
        }

        Ok(Lsps1Options {
            min_required_channel_confirmations,
            min_funding_confirms_within_blocks,
            min_onchain_payment_confirmations,
            supports_zero_channel_reserve,
            min_onchain_payment_size_sat,
            max_channel_expiry_blocks,
            min_initial_client_balance_sat,
            max_initial_client_balance_sat,
            min_initial_lsp_balance_sat,
            max_initial_lsp_balance_sat,
            min_channel_balance_sat,
            max_channel_balance_sat,
        })
    }
}

#[derive(Default, Debug)]
pub struct Lsps1CreateOrderRequestBuilder {
    lsp_balance_sat: Option<SatAmount>,
    client_balance_sat: Option<SatAmount>,
    funding_confirms_within_blocks: Option<u8>,
    channel_expiry_blocks: Option<u32>,
    token: Option<String>,
    refund_onchain_address: Option<OnchainAddress>,
    announce_channel: Option<bool>,
}

impl Lsps1CreateOrderRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lsp_balance_sat(mut self, lsp_balance_sat: SatAmount) -> Self {
        self.lsp_balance_sat = Some(lsp_balance_sat);
        self
    }

    pub fn channel_expiry_blocks(mut self, channel_expiry_blocks: u32) -> Self {
        self.channel_expiry_blocks = Some(channel_expiry_blocks);
        self
    }

    pub fn client_balance_sat(mut self, client_balance_sat: Option<SatAmount>) -> Self {
        self.client_balance_sat = client_balance_sat;
        self
    }

    pub fn funding_confirms_within_blocks(mut self, funding_confirms_within_blocks: Option<u8>) -> Self {
        self.funding_confirms_within_blocks = funding_confirms_within_blocks;
        self
    }

    pub fn token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    pub fn refund_onchain_address(
        mut self,
        refund_onchain_address: Option<OnchainAddress>,
    ) -> Self {
        self.refund_onchain_address = refund_onchain_address;
        self
    }

    pub fn announce_channel(mut self, announce_channel: Option<bool>) -> Self {
        self.announce_channel = announce_channel;
        self
    }

    pub fn build(self) -> Result<Lsps1CreateOrderRequest> {
        // Required fields
        let lsp_balance_sat = self
            .lsp_balance_sat
            .context("Missing field 'lsp_balance_sat' in Lsps1CreateOrderRequestBuilder")?;
        let channel_expiry_blocks = self
            .channel_expiry_blocks
            .context("Missing field 'channel_expirty_blocks' in Lsps1CreateOrderRequestBuilder")?;

        // Fields that allow for reasonable defaults
        let client_balance_sat = self.client_balance_sat.unwrap_or(SatAmount::new(0));
        let announce_channel = self.announce_channel.unwrap_or(false);
        let funding_confirms_within_blocks = self.funding_confirms_within_blocks.unwrap_or(6);

        // Non-required fields
        let token = self.token;
        let refund_onchain_address = self.refund_onchain_address;

        let request = Lsps1CreateOrderRequest {
            lsp_balance_sat,
            client_balance_sat,
            funding_confirms_within_blocks,
            channel_expiry_blocks,
            token,
            refund_onchain_address,
            announce_channel,
        };

        Ok(request)
    }
}

#[derive(Default, Debug)]
pub struct Lsps1CreateOrderResponseBuilder {
    uuid: Option<Uuid>,
    lsp_balance_sat: Option<SatAmount>,
    client_balance_sat: Option<SatAmount>,
    funding_confirms_within_blocks: Option<u8>,
    channel_expiry_blocks: Option<u32>,
    token: Option<String>,
    announce_channel: Option<bool>,
    created_at: Option<IsoDatetime>,
    expires_at: Option<IsoDatetime>,
    order_state: Option<OrderState>,
    payment: Option<Payment>,
    channel: Option<Channel>,
}

impl Lsps1CreateOrderResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_request(request: Lsps1CreateOrderRequest) -> Self {
        Self::new()
            .lsp_balance_sat(request.lsp_balance_sat)
            .client_balance_sat(request.client_balance_sat)
            .funding_confirms_within_blocks(request.funding_confirms_within_blocks)
            .channel_expiry_blocks(request.channel_expiry_blocks)
            .token(request.token.unwrap_or("".to_string()))
            // .refund_onchain_address(request.refund_onchain_address)
            .announce_channel(request.announce_channel)
    }

    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }
    pub fn lsp_balance_sat(mut self, lsp_balance_sat: SatAmount) -> Self {
        self.lsp_balance_sat = Some(lsp_balance_sat);
        self
    }
    pub fn client_balance_sat(mut self, client_balance_sat: SatAmount) -> Self {
        self.client_balance_sat = Some(client_balance_sat);
        self
    }
    pub fn funding_confirms_within_blocks(mut self, funding_confirms_within_blocks: u8) -> Self {
        self.funding_confirms_within_blocks = Some(funding_confirms_within_blocks);
        self
    }
    pub fn channel_expiry_blocks(mut self, channel_expiry_blocks: u32) -> Self {
        self.channel_expiry_blocks = Some(channel_expiry_blocks);
        self
    }
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
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
    pub fn order_state(mut self, order_state: OrderState) -> Self {
        self.order_state = Some(order_state);
        self
    }
    pub fn payment(mut self, payment: Payment) -> Self {
        self.payment = Some(payment);
        self
    }
    pub fn channel(mut self, channel: Option<Channel>) -> Self {
        self.channel = channel;
        self
    }

    pub fn build(self) -> Result<Lsps1CreateOrderResponse> {
        //required variables
        let order_id = self
            .uuid
            .context("Missing field 'order_id' in Lsps1CreateOrderRequestBuilder")?;
        let lsp_balance_sat = self
            .lsp_balance_sat
            .context("Missing field 'lsp_balance_sat' in Lsps1CreateOrderRequestBuilder")?;
        let client_balance_sat = self
            .client_balance_sat
            .context("Missing field 'client_balance_sat' in Lsps1CreateOrderRequestBuilder")?;
        let funding_confirms_within_blocks = self
            .funding_confirms_within_blocks
            .context("Missing field 'funding_confirms_within_blocks' in Lsps1CreateOrderRequestBuilder")?;
        let channel_expiry_blocks = self
            .channel_expiry_blocks
            .context("Missing field 'channel_expiry_blocks' in Lsps1CreateOrderRequestBuilder")?;
        let token = self.token.unwrap_or("".to_string());
        let announce_channel = self
            .announce_channel
            .context("Missing field 'announce_channel' in Lsps1CreateOrderRequestBuilder")?;
        let created_at = self
            .created_at
            .context("Missing field 'created_at' in Lsps1CreateOrderRequestBuilder")?;
        let order_state = self
            .order_state
            .context("Missing field 'order_state' in Lsps1CreateOrderRequestBuilder")?;
        let expires_at = self
            .expires_at
            .context("Missing field 'expires_at' in Lsps1CreateOrderRequestBuilder")?;
        let payment = self
            .payment
            .context("Missing field 'payment' in Lsps1CreateOrderRequestBuilder")?;
        let channel = self.channel;

        let request = Lsps1CreateOrderResponse {
            order_id,
            lsp_balance_sat,
            client_balance_sat,
            funding_confirms_within_blocks,
            channel_expiry_blocks,
            token,
            announce_channel,
            created_at,
            expires_at,
            order_state,
            payment,
            channel,
        };

        Ok(request)
    }
}

#[derive(Default, Debug)]
pub struct OnchainPaymentBuilder {
    outpoint: Option<String>,
    sat: Option<SatAmount>,
    confirmed: Option<bool>,
}

impl OnchainPaymentBuilder {
    pub fn outpoint(mut self, outpoint: String) -> Self {
        self.outpoint = Some(outpoint);
        self
    }

    pub fn sat(mut self, sat: SatAmount) -> Self {
        self.sat = Some(sat);
        self
    }

    pub fn confirmed(mut self, confirmed: bool) -> Self {
        self.confirmed = Some(confirmed);
        self
    }

    pub fn build(self) -> Result<OnchainPayment> {
        let outpoint = self.outpoint.context("Missing field 'outpoint'")?;
        let sat = self.sat.context("Missing field 'sat'")?;
        let confirmed = self.confirmed.context("Missing field 'confirmed'")?;

        let payment = OnchainPayment {
            outpoint,
            sat,
            confirmed,
        };

        Ok(payment)
    }
}

#[derive(Default, Debug)]
pub struct PaymentBuilder {
    state: Option<PaymentState>,
    fee_total_sat: Option<SatAmount>,
    order_total_sat: Option<SatAmount>,

    bolt11_invoice: Option<String>,
    onchain_address: Option<OnchainAddress>,
    required_onchain_block_confirmations: Option<u8>,

    minimum_fee_for_0conf: Option<FeeRate>,
    onchain_payment: Option<OnchainPayment>,
}

#[derive(Default, Debug)]
pub struct Lsps1GetOrderRequestBuilder {
    order_id: Option<String>,
}

impl Lsps1GetOrderRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn order_id(mut self, order_id: String) -> Self {
        self.order_id = Some(order_id);
        self
    }

    pub fn build(self) -> Result<Lsps1GetOrderRequest> {
        Ok(Lsps1GetOrderRequest {
            order_id: self
                .order_id
                .context("Missing field 'order_id' in Lsps1GetOrderRequestBuilder")?,
        })
    }
}

impl PaymentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(mut self, state: PaymentState) -> Self {
        self.state = Some(state);
        self
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

    pub fn onchain_address(mut self, onchain_address: Option<OnchainAddress>) -> Self {
        self.onchain_address = onchain_address;
        self
    }

    pub fn required_onchain_block_confirmations(
        mut self,
        required_onchain_block_confirmations: u8,
    ) -> Self {
        self.required_onchain_block_confirmations = Some(required_onchain_block_confirmations);
        self
    }

    pub fn minimum_fee_for_0conf(mut self, minimum_fee_for_0conf: Option<FeeRate>) -> Self {
        self.minimum_fee_for_0conf = minimum_fee_for_0conf;
        self
    }

    pub fn onchain_payment(mut self, onchain_payment: OnchainPayment) -> Self {
        self.onchain_payment = Some(onchain_payment);
        self
    }

    pub fn build(self) -> Result<Payment> {
        // Required fields
        let state = self.state.context("Missing field 'state'")?;
        let fee_total_sat = self
            .fee_total_sat
            .context("Missing field 'fee_total_sat'")?;
        let order_total_sat = self
            .order_total_sat
            .context("Missing field 'order_total_sat'")?;
        let bolt11_invoice = self
            .bolt11_invoice
            .context("Missing field 'bolt11_invoice'")?;

        // Optional parameters
        let onchain_address = self.onchain_address;
        let required_onchain_block_confirmations = self.required_onchain_block_confirmations;
        let minimum_fee_for_0conf = self.minimum_fee_for_0conf;
        let onchain_payment = self.onchain_payment;

        if onchain_address.is_none() {
            if required_onchain_block_confirmations.is_some() {
                return Err(anyhow!("'required_onchain_block_confirmations' expected to be null because 'onchain_address' is null"));
            }

            if minimum_fee_for_0conf.is_some() {
                return Err(anyhow!(
                    "'minimum_fee_for_0conf' expected to be null because 'onchain_address' is null"
                ));
            }
        }

        let payment = Payment {
            state,
            fee_total_sat,
            order_total_sat,
            bolt11_invoice,
            onchain_address,
            min_onchain_payment_confirmations: required_onchain_block_confirmations,
            min_fee_for_0conf: minimum_fee_for_0conf,
            onchain_payment,
        };

        Ok(payment)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn options_cannot_be_constructed_if_required_parameters_are_missing() {
        let _ = Lsps1OptionsBuilder::new()
            .build()
            .expect_err("Should fail because a required parameter is missing");
    }

    #[test]
    fn options_can_be_constructed_when_all_params_are_specified() {
        // I picked unique values on each field to test that I am not switching
        // values by accident
        let lsps1_options = Lsps1OptionsBuilder::new()
            .min_required_channel_confirmations(6)
            .min_onchain_payment_confirmations(None)
            .min_funding_confirms_within_blocks(6)
            .supports_zero_channel_reserve(false)
            .max_channel_expiry_blocks(5)
            .min_onchain_payment_size_sat(Some(SatAmount::new(10_000)))
            .min_channel_balance_sat(SatAmount::new(50_001))
            .max_channel_balance_sat(SatAmount::new(100_001))
            .min_initial_client_balance_sat(SatAmount::new(0))
            .max_initial_client_balance_sat(SatAmount::new(1))
            .min_initial_lsp_balance_sat(SatAmount::new(2))
            .max_initial_lsp_balance_sat(SatAmount::new(100_003))
            .build()
            .unwrap();

        assert_eq!(lsps1_options.min_required_channel_confirmations, 6);
        assert_eq!(lsps1_options.min_onchain_payment_confirmations, None);
        assert_eq!(lsps1_options.min_funding_confirms_within_blocks, 6);
        assert!(!lsps1_options.supports_zero_channel_reserve);
        assert_eq!(lsps1_options.max_channel_expiry_blocks, 5);
        assert_eq!(
            lsps1_options.min_onchain_payment_size_sat.unwrap(),
            SatAmount::new(10_000)
        );
        assert_eq!(
            lsps1_options.min_channel_balance_sat,
            SatAmount::new(50_001)
        );
        assert_eq!(
            lsps1_options.max_channel_balance_sat,
            SatAmount::new(100_001)
        );
        assert_eq!(
            lsps1_options.min_initial_client_balance_sat,
            SatAmount::new(0)
        );
        assert_eq!(
            lsps1_options.max_initial_client_balance_sat,
            SatAmount::new(1)
        );
        assert_eq!(lsps1_options.min_initial_lsp_balance_sat, SatAmount::new(2));
        assert_eq!(
            lsps1_options.max_initial_lsp_balance_sat,
            SatAmount::new(100_003)
        )
    }

    #[test]
    fn options_can_be_constructed_without_optional_params() {
        let lsps1_options = Lsps1OptionsBuilder::new()
            .min_required_channel_confirmations(6)
            .supports_zero_channel_reserve(false)
            .min_funding_confirms_within_blocks(6)
            .max_channel_expiry_blocks(5)
            .min_channel_balance_sat(SatAmount::new(50_001))
            .max_channel_balance_sat(SatAmount::new(100_001))
            .min_initial_client_balance_sat(SatAmount::new(0))
            .max_initial_client_balance_sat(SatAmount::new(1))
            .min_initial_lsp_balance_sat(SatAmount::new(2))
            .max_initial_lsp_balance_sat(SatAmount::new(100_003))
            .build()
            .unwrap();

        assert_eq!(lsps1_options.min_onchain_payment_confirmations, None);
        assert_eq!(lsps1_options.min_onchain_payment_size_sat, None);
    }

    #[test]
    fn options_min_channel_balance_should_be_less_than_max() {
        let error = Lsps1OptionsBuilder::new()
            .min_required_channel_confirmations(6)
            .min_funding_confirms_within_blocks(6)
            .supports_zero_channel_reserve(false)
            .max_channel_expiry_blocks(5)
            .min_channel_balance_sat(SatAmount::new(100_001)) // Max > min
            .max_channel_balance_sat(SatAmount::new(50_001))
            .min_initial_client_balance_sat(SatAmount::new(0))
            .max_initial_client_balance_sat(SatAmount::new(1))
            .min_initial_lsp_balance_sat(SatAmount::new(2))
            .max_initial_lsp_balance_sat(SatAmount::new(100_003))
            .build()
            .expect_err("Should fail because max_channel_balance_sat < min_channel_balance_sat");

        assert!(format!("{:?}", error).contains("channel_balance_sat"))
    }

    #[test]
    fn options_min_initial_client_balance_sat_should_be_less_than_max() {
        let error = Lsps1OptionsBuilder::new()
            .min_required_channel_confirmations(6)
            .min_funding_confirms_within_blocks(0)
            .supports_zero_channel_reserve(false)
            .max_channel_expiry_blocks(5)
            .min_channel_balance_sat(SatAmount::new(10_000))
            .max_channel_balance_sat(SatAmount::new(50_000))
            .min_initial_client_balance_sat(SatAmount::new(2)) // Spot the error
            .max_initial_client_balance_sat(SatAmount::new(1)) // Spot the error
            .min_initial_lsp_balance_sat(SatAmount::new(2))
            .max_initial_lsp_balance_sat(SatAmount::new(3))
            .build()
            .expect_err("Should fail because max_initial_client_balance_sat < min_initial_client_balance_sat");

        assert!(format!("{:?}", error).contains("max_initial_client_balance_sat"),)
    }

    #[test]
    fn options_min_initial_lsp_balance_sat_should_be_less_than_max() {
        let error = Lsps1OptionsBuilder::new()
            .min_required_channel_confirmations(6)
            .min_funding_confirms_within_blocks(0)
            .supports_zero_channel_reserve(false)
            .max_channel_expiry_blocks(5)
            .min_channel_balance_sat(SatAmount::new(10_000))
            .max_channel_balance_sat(SatAmount::new(50_000))
            .min_initial_client_balance_sat(SatAmount::new(1))
            .max_initial_client_balance_sat(SatAmount::new(5))
            .min_initial_lsp_balance_sat(SatAmount::new(3)) // Spot the error
            .max_initial_lsp_balance_sat(SatAmount::new(1)) // Spot the error
            .build()
            .expect_err(
                "Should fail because max_initial_lsp_balance_sat < min_initial_lsp_balance_sat",
            );

        assert!(
            format!("{:?}", error).contains("max_initial_lsp_balance_sat"),
            "Error should contain max_initial_lsp_balance_sat"
        )
    }
}
