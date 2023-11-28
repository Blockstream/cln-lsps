use anyhow::Result;
use crate::lsps0::common_schemas::NetworkValidation;
use crate::lsps1::schema::{Lsps1CreateOrderRequest, Lsps1Options};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Lsps1OptionMismatchError {
    property : String,
    message : String
}

impl Lsps1OptionMismatchError {

    fn new(property : String, message : String) -> Self {
        Self {property, message}
    }
}


impl<V : NetworkValidation> Lsps1CreateOrderRequest<V> {

    pub fn validate_options(&self, options : &Lsps1Options) -> Result<(), Lsps1OptionMismatchError> {

        if self.client_balance_sat < options.min_initial_client_balance_sat {
            return Err(Lsps1OptionMismatchError::new(
                "min_initial_client_balance_sat".to_string(), 
                format!("You've requested client_balance_sat={} but the LSP-server requires at least {}",
                        self.client_balance_sat, 
                        options.min_initial_client_balance_sat)))
        }

        if self.client_balance_sat > options.max_initial_client_balance_sat {
            return Err(Lsps1OptionMismatchError::new(
                "max_initial_client_balance_sat".to_string(), 
                format!("You've requested client_balance_sat={} but the LSP-server doesn't allow this value to exceed {}",
                        self.client_balance_sat, 
                        options.max_initial_client_balance_sat)))
        }

        if self.lsp_balance_sat < options.min_initial_lsp_balance_sat {
            return Err(Lsps1OptionMismatchError::new(
                    "min_initial_lsp_balance_sat".to_string(),
                    format!("You've requested a channel with lsp_balance_sat={} but the LSP-server requires at least {}",
                            self.lsp_balance_sat,
                            options.min_initial_lsp_balance_sat)))
        }

        if self.lsp_balance_sat > options.max_initial_lsp_balance_sat {
            return Err(Lsps1OptionMismatchError::new(
                    "max_initial_lsp_balance_sat".to_string(),
                    format!("You've requested a channel with lsp_balance_sat={} but the LSP-server doesn't allow this value to exceed {}",
                            self.lsp_balance_sat,
                            options.max_initial_lsp_balance_sat)))
        }

        // Compute the capacity of the channel and validate it
        // The checked_add returns None on overflow
        let capacity_option = self.lsp_balance_sat.checked_add(&self.client_balance_sat);
        let capacity = match capacity_option {
            Some(c) => c,
            None => {
                return Err(Lsps1OptionMismatchError::new(
                        "max_channel_balance_sat".to_string(),
                        format!("Overflow when computing channel_capacity")
                    ));}
        };

        if capacity < options.min_channel_balance_sat {
            return Err(Lsps1OptionMismatchError::new(
                    "min_channel_balance_sat".to_string(),
                    format!("You've requested a channel with capacity={} but the LSP-server requires at least {}", 
                            capacity,
                            options.min_channel_balance_sat
                    )))
        }

        if capacity > options.max_channel_balance_sat {
            return Err(Lsps1OptionMismatchError::new(
                    "max_channel_balance_sat".to_string(),
                    format!("You've requested a channel with capacity={} but the LSP-server only allows values up to {}",
                            capacity,
                            options.max_channel_balance_sat
                            )))
        }

        // Verify the channel_expiry_blocks
        if self.channel_expiry_blocks > options.max_channel_expiry_blocks {
            return Err(Lsps1OptionMismatchError::new(
                    "max_channel_expiry_blocks".to_string(),
                    format!("You've requested to lease a channel for channel_expiry_block={} but the LSP-server only allows max_channel_expiry_blocks={}",
                            self.channel_expiry_blocks,
                            options.max_channel_expiry_blocks
                            )))
        }
        return Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::lsps0::common_schemas::SatAmount;
    use crate::lsps1::builders::{Lsps1OptionsBuilder,Lsps1CreateOrderRequestBuilder};

    fn get_options_builder() -> Lsps1OptionsBuilder {
        Lsps1OptionsBuilder::new()
            .minimum_channel_confirmations(0)
            .minimum_onchain_payment_confirmations(None)
            .supports_zero_channel_reserve(true)
            .min_onchain_payment_size_sat(None)
            .max_channel_expiry_blocks(1_000)
            .min_initial_client_balance_sat(SatAmount::new(0))
            .max_initial_client_balance_sat(SatAmount::new(0))
            .min_initial_lsp_balance_sat(SatAmount::new(100_000))
            .max_initial_lsp_balance_sat(SatAmount::new(100_000_000))
            .min_channel_balance_sat(SatAmount::new(100_000))
            .max_channel_balance_sat(SatAmount::new(100_000_000))
    }

    fn get_order_builder() -> Lsps1CreateOrderRequestBuilder {
        Lsps1CreateOrderRequestBuilder::new()
            .api_version(1)
            .client_balance_sat(SatAmount::new(0))
            .lsp_balance_sat(SatAmount::new(500_000))
            .confirms_within_blocks(6)
            .channel_expiry_blocks(1_000)
            .token(None)
            .refund_onchain_address(None)
            .announce_channel(false)
    }

    #[test]
    fn test_validate_order_against_options_okay() {
        let options = get_options_builder().build().unwrap();
        let order = get_order_builder().build().unwrap();

        // Should be valid
        order.validate_options(&options).unwrap();
    }

    #[test]
    fn test_validate_order_against_options_lsp_balance_sat_error() {

        let options = get_options_builder()
            .min_initial_lsp_balance_sat(SatAmount::new(1_000))
            .max_initial_lsp_balance_sat(SatAmount::new(100_000))
            .build().unwrap();

        let order_1 = get_order_builder()
            .lsp_balance_sat(SatAmount::new(0))
            .build()
            .unwrap();

        let order_2 = get_order_builder()
            .lsp_balance_sat(SatAmount::new(999_999))
            .build()
            .unwrap();

        let err1 = order_1.validate_options(&options).unwrap_err();
        let err2 = order_2.validate_options(&options).unwrap_err();

        assert_eq!(err1.property, "min_initial_lsp_balance_sat");
        assert_eq!(err2.property, "max_initial_lsp_balance_sat");

    }

    #[test]
    fn test_validate_order_against_options_client_balance_sat_error() {
        let options = get_options_builder()
            .min_initial_client_balance_sat(SatAmount::new(1_000))
            .max_initial_client_balance_sat(SatAmount::new(100_000))
            .build().unwrap();

        let order_1 = get_order_builder()
            .client_balance_sat(SatAmount::new(0))
            .build()
            .unwrap();

        let order_2 = get_order_builder()
            .client_balance_sat(SatAmount::new(999_999))
            .build()
            .unwrap();

        let err1 = order_1.validate_options(&options).unwrap_err();
        let err2 = order_2.validate_options(&options).unwrap_err();

        assert_eq!(err1.property, "min_initial_client_balance_sat");
        assert_eq!(err2.property, "max_initial_client_balance_sat");
    }

    #[test]
    fn test_validate_order_against_capacity_from_options_error() {
        let options = get_options_builder()
            .min_initial_client_balance_sat(SatAmount::new(1_000))
            .max_initial_client_balance_sat(SatAmount::new(100_000))
            .min_initial_lsp_balance_sat(SatAmount::new(1_000))
            .max_initial_lsp_balance_sat(SatAmount::new(100_000))
            .min_channel_balance_sat(SatAmount::new(10_000))
            .max_channel_balance_sat(SatAmount::new(100_000))
            .build().unwrap();

        let order_1 = get_order_builder()
            .client_balance_sat(SatAmount::new(1_000))
            .lsp_balance_sat(SatAmount::new(1_000))
            .build()
            .unwrap();

        let order_2 = get_order_builder()
            .client_balance_sat(SatAmount::new(100_000))
            .lsp_balance_sat(SatAmount::new(100_000))
            .build()
            .unwrap();

        let err1 = order_1.validate_options(&options).unwrap_err();
        let err2 = order_2.validate_options(&options).unwrap_err();

        assert_eq!(err1.property, "min_channel_balance_sat");
        assert_eq!(err2.property, "max_channel_balance_sat");
    }

}
