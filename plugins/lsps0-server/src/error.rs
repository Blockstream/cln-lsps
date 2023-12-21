use lsp_primitives::json_rpc::{DefaultError, ErrorData};
use lsp_primitives::lsps1::util::Lsps1OptionMismatchError;

// TODO: Find a better name
#[derive(Debug, Clone)]
pub enum InvoicePaymentError {
    /// The handler recognizes the invoice and actively rejects it
    PaymentRejected(String),
    /// The handler encountered an error
    /// The error will be logged
    Log(String),
    /// The handler ignores the payment
    NotMine,
}
