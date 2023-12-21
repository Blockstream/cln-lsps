use serde::{Serialize, Deserialize};
use crate::json_rpc::{JsonRpcId, JsonRpcResponseFailure};

pub mod codes {
    pub const PARSE_ERROR_CODE : i64 = -32700;
    pub const PARSE_ERROR_MSG : &str = "Parse Error";

    pub const INVALID_REQUEST_CODE : i64 = -36200;
    pub const INVALID_REQUEST_MSG : &str = "Invalid Request";

    pub const METHOD_NOT_FOUND_CODE : i64 = -36201;
    pub const METHOD_NOT_FOUND_MSG : &str = "Method not found";

    pub const INVALID_PARAMS_CODE : i64 = -36202;
    pub const INVALID_PARAMS_MSG : &str = "Invalid Params";

    pub const INTERNAL_ERROR_CODE : i64 = -32603;
    pub const INTERNAL_ERROR_MSG : &str = "Internal Error";

    pub const NOT_FOUND_CODE : i64 = 404;
    pub const NOT_FOUND_MSG : &str = "Not Found";

    pub const OPTIONS_MISMATCH_CODE : i64 = 1000;
    pub const OPTIONS_MISMATCH_MSG : &str = "Options mismatch";

    pub const CLIENT_REJECTED_CODE : i64 = 1001;
    pub const CLIENT_REJECTED_MSG : &str = "Client rejected";
}

pub type DefaultError = serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorData<E = DefaultError> {
    pub code: i64,
    pub message: String,
    pub data: Option<E>,
}

impl<E> ErrorData<E> {
    pub fn into_response<O>(self, id: JsonRpcId) -> JsonRpcResponseFailure<E> {
        JsonRpcResponseFailure {
            id,
            jsonrpc: String::from("2.0"),
            error: self,
        }
    }
}

impl ErrorData<DefaultError> {
    pub fn parse_error(_message: String) -> Self {
        Self {
            code: codes::PARSE_ERROR_CODE,
            message: String::from(codes::PARSE_ERROR_MSG),
            data: None,
        }
    }

    pub fn invalid_request(_message: String) -> Self {
        Self {
            code: codes::INVALID_REQUEST_CODE,
            message: codes::INVALID_REQUEST_MSG.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: codes::METHOD_NOT_FOUND_CODE,
            message: codes::METHOD_NOT_FOUND_MSG.into(),
            data: Some(serde_json::json!({"method" : method})),
        }
    }

    pub fn not_found() -> Self {
        Self {
            code: 404,
            message: String::from("Not Found"),
            data: None,
        }
    }
}

impl<E> ErrorData<E> {
    pub fn invalid_params(data: E) -> Self {
        Self {
            code: codes::INVALID_PARAMS_CODE,
            message: codes::INVALID_PARAMS_MSG.into(),
            data: Some(data),
        }
    }
}

impl<E> ErrorData<E> {
    pub fn internal_error(data: E) -> Self {
        Self {
            code: codes::INTERNAL_ERROR_CODE,
            message: codes::INTERNAL_ERROR_MSG.into(),
            data: Some(data),
        }
    }
}

impl ErrorData<DefaultError> {
    pub fn internalize<T: core::fmt::Debug>(err: T) -> Self {
        Self::internal_error(serde_json::Value::String(format!("{:?}", err)))
    }
}