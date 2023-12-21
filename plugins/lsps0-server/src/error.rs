use lsp_primitives::json_rpc::{DefaultError, ErrorData};
use lsp_primitives::lsps1::util::Lsps1OptionMismatchError;

#[derive(Debug, Clone)]
pub enum CustomMsgError {
    InvalidParams(Box<str>),
    InternalError(Box<str>),
    UnknownMethod(Box<str>),
    Lsps1OptionMismatch(Lsps1OptionMismatchError),
    NotFound(Box<str>),
}

impl TryFrom<CustomMsgError> for ErrorData<DefaultError> {
    type Error = anyhow::Error;

    fn try_from(error: CustomMsgError) -> Result<Self, Self::Error> {
        match error {
            CustomMsgError::InternalError(_) => Ok(ErrorData::internal_error()),
            CustomMsgError::InvalidParams(params) => {
                Ok(ErrorData::invalid_params(serde_json::json!({"message" : params})))
            }
            CustomMsgError::UnknownMethod(method) => Ok(ErrorData::unknown_method(&method)),
            CustomMsgError::Lsps1OptionMismatch(om) => ErrorData::try_from(om),
            CustomMsgError::NotFound(msg) => {
                let default_error = serde_json::json!({"message" : msg});
                Ok(ErrorData {
                    code: 404,
                    message: "Not Found".into(),
                    data: Some(default_error),
                })
            }
        }
    }
}

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
