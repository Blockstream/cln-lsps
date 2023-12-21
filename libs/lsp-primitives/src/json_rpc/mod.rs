use base64::Engine as _;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::lsps0::parameter_validation;
pub use crate::no_params::NoParams;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum JsonRpcId {
    String(String),
    Number(i64),
    None,
}

impl PartialEq<serde_json::Value> for JsonRpcId {
    fn eq(&self, value: &serde_json::Value) -> bool {
        match self {
            Self::String(s) => value == s,
            Self::Number(n) => value == n,
            Self::None => value == &serde_json::Value::Null,
        }
    }
}

impl PartialEq<&str> for JsonRpcId {
    fn eq(&self, value: &&str) -> bool {
        match self {
            Self::String(s) => s == value,
            Self::Number(_) => false,
            Self::None => false,
        }
    }
}

impl From<&str> for JsonRpcId {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl FromStr for JsonRpcId {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::String(value.to_string()))
    }
}

/// Generate a random json_rpc_id string that follows the requirements of LSPS0
///
/// - Should be a String
/// - Should be at generated using at least 80 bits of randomness
pub fn generate_random_rpc_id() -> JsonRpcId {
    // The specification requires an id using least 80 random bits of randomness
    let seed: [u8; 10] = rand::random();
    let str_id = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(seed);
    JsonRpcId::String(str_id)
}

/// Defines a json-rpc method and describes the schema
/// of the input I, output O and error-type E.
///
/// The error-type can be serialized and deserialized
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct JsonRpcMethod<'a, I, O, E> {
    pub method: &'a str,
    #[serde(skip_serializing)]
    request: std::marker::PhantomData<I>,
    #[serde(skip_serializing)]
    return_type: std::marker::PhantomData<O>,
    #[serde(skip_serializing)]
    error_type: std::marker::PhantomData<E>,
}

impl<'a, I, O, E> JsonRpcMethod<'a, I, O, E> {
    pub const fn new(method: &'a str) -> Self {
        Self {
            method,
            request: std::marker::PhantomData,
            return_type: std::marker::PhantomData,
            error_type: std::marker::PhantomData,
        }
    }

    pub const fn name(&self) -> &'a str {
        self.method
    }

    /// Creates a JsonRpcRequest with parameters for the given method
    pub fn create_request(&self, params: I, json_rpc_id: JsonRpcId) -> JsonRpcRequest<I> {
        JsonRpcRequest::<I> {
            jsonrpc: String::from("2.0"),
            id: json_rpc_id,
            method: self.method.into(),
            params,
        }
    }

    pub fn create_ok_response(
        &self,
        request: JsonRpcRequest<I>,
        result: O,
    ) -> JsonRpcResponse<O, E> {
        JsonRpcResponse::Ok(JsonRpcResponseSuccess {
            jsonrpc: String::from("2.0"),
            id: request.id.clone(),
            result,
        })
    }

    pub fn into_typed_request(
        &self,
        request: JsonRpcRequest<serde_json::Value>,
    ) -> Result<JsonRpcRequest<I>, ErrorData>
    where
        I: DeserializeOwned + parameter_validation::ExpectedFields,
    {
        let params = request.params;
        let params: I = parameter_validation::from_value(params)?;

        let request = JsonRpcRequest::<I> {
            id: request.id,
            jsonrpc: request.jsonrpc,
            method: request.method,
            params: params,
        };

        Ok(request)
    }
}

impl<O, E> JsonRpcMethod<'_, NoParams, O, E> {
    pub fn create_request_no_params(&self, json_rpc_id: JsonRpcId) -> JsonRpcRequest<NoParams> {
        self.create_request(NoParams, json_rpc_id)
    }
}

impl<'a, I, O, E> std::convert::From<&'a JsonRpcMethod<'a, I, O, E>> for String {
    fn from(value: &JsonRpcMethod<I, O, E>) -> Self {
        value.method.into()
    }
}

impl<'de, I, O, E> JsonRpcMethod<'de, I, O, E>
where
    O: Deserialize<'de>,
    E: Deserialize<'de>,
{
    pub fn parse_json_response_str(
        &self,
        json_str: &'de str,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

impl<I, O, E> JsonRpcMethod<'_, I, O, E>
where
    O: DeserializeOwned,
    E: DeserializeOwned,
{
    pub fn parse_json_response_value(
        &self,
        json_value: serde_json::Value,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error> {
        serde_json::from_value(json_value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest<I> {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub method: String,
    pub params: I,
}

impl<I> JsonRpcRequest<I> {
    pub fn new<O, E>(method: JsonRpcMethod<I, O, E>, params: I) -> Self {
        Self {
            jsonrpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: method.method.into(),
            params,
        }
    }
}

impl JsonRpcRequest<serde_json::Value> {
    pub fn deserialize<I>(self) -> Result<JsonRpcRequest<I>, serde_json::Error>
    where
        I: DeserializeOwned,
    {
        let request = JsonRpcRequest {
            jsonrpc: self.jsonrpc,
            id: self.id,
            method: self.method,
            params: serde_json::from_value(self.params)?,
        };
        Ok(request)
    }
}

impl JsonRpcRequest<NoParams> {
    pub fn new_no_params<O, E>(method: JsonRpcMethod<NoParams, O, E>) -> Self {
        Self {
            jsonrpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: method.method.into(),
            params: NoParams,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponseSuccess<O> {
    pub id: JsonRpcId,
    pub result: O,
    pub jsonrpc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponseFailure<E> {
    pub id: JsonRpcId,
    pub error: ErrorData<E>,
    pub jsonrpc: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse<O, E> {
    Error(JsonRpcResponseFailure<E>),
    Ok(JsonRpcResponseSuccess<O>),
}

impl<E, O> JsonRpcResponse<E, O> {
    pub fn jsonrpc(&self) -> &str {
        match self {
            JsonRpcResponse::Ok(j) => &j.jsonrpc,
            JsonRpcResponse::Error(j) => &j.jsonrpc,
        }
    }

    pub fn id(&self) -> &JsonRpcId {
        match self {
            JsonRpcResponse::Ok(j) => &j.id,
            JsonRpcResponse::Error(j) => &j.id,
        }
    }
}

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
            code: -32700,
            message: String::from("parse_error"),
            data: None,
        }
    }

    pub fn invalid_request(_message: String) -> Self {
        Self {
            code: -32600,
            message: String::from("invalid_request"),
            data: None,
        }
    }

    pub fn unknown_method(method: &str) -> Self {
        Self {
            code: -32601,
            message: String::from("unknown_method"),
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
            code: -32602,
            message: String::from("invalid_params"),
            data: Some(data),
        }
    }
}

impl<E> ErrorData<E> {
    pub fn internal_error(data: E) -> Self {
        Self {
            code: -32603,
            message: String::from("error_data"),
            data: Some(data),
        }
    }
}

impl ErrorData<DefaultError> {
    pub fn internalize<T: core::fmt::Debug>(err: T) -> Self {
        Self::internal_error(serde_json::Value::String(format!("{:?}", err)))
    }
}

impl<O, E> JsonRpcResponse<O, E> {
    pub fn success(id: JsonRpcId, output: O) -> Self {
        let success = JsonRpcResponseSuccess {
            id,
            result: output,
            jsonrpc: String::from("2.0"),
        };

        JsonRpcResponse::Ok(success)
    }

    pub fn error(id: JsonRpcId, error: ErrorData<E>) -> Self {
        let error = JsonRpcResponseFailure {
            id,
            error,
            jsonrpc: String::from("2.0"),
        };

        JsonRpcResponse::Error(error)
    }
}

pub type DefaultError = serde_json::Value;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn serialize_json_rpc_request() {
        let rpc_request = JsonRpcRequest {
            id: "abcefg".into(),
            jsonrpc: "2.0".into(),
            params: NoParams,
            method: "test.method".into(),
        };

        let json_str = serde_json::to_string(&rpc_request).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value.get("jsonrpc").unwrap(), "2.0");
        assert_eq!(&rpc_request.id, value.get("id").unwrap());
        assert_eq!(value.get("method").unwrap(), "test.method");
        assert!(value.get("params").unwrap().as_object().unwrap().is_empty())
    }

    #[test]
    fn serialize_json_rpc_response_success() {
        let rpc_response_ok: JsonRpcResponseSuccess<String> = JsonRpcResponseSuccess {
            id: JsonRpcId::String("abc".to_string()),
            result: String::from("result_data"),
            jsonrpc: String::from("2.0"),
        };

        let rpc_response: JsonRpcResponse<String, ()> = JsonRpcResponse::Ok(rpc_response_ok);

        let json_str: String = serde_json::to_string(&rpc_response).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value.get("jsonrpc").unwrap(), "2.0");
        assert_eq!(value.get("id").unwrap(), "abc");
        assert_eq!(value.get("result").unwrap(), "result_data")
    }

    #[test]
    fn serialize_json_rpc_response_error() {
        let rpc_response: JsonRpcResponse<String, ()> =
            JsonRpcResponse::Error(JsonRpcResponseFailure {
                jsonrpc: String::from("2.0"),
                id: JsonRpcId::String("abc".to_string()),
                error: ErrorData {
                    code: -32700,
                    message: String::from("Failed to parse data"),
                    data: None,
                },
            });

        let json_str: String = serde_json::to_string(&rpc_response).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value.get("jsonrpc").unwrap(), "2.0");
        assert_eq!(value.get("id").unwrap(), "abc");
        assert_eq!(value.get("error").unwrap().get("code").unwrap(), -32700);
        assert_eq!(
            value.get("error").unwrap().get("message").unwrap(),
            "Failed to parse data"
        );
    }

    #[test]
    fn create_rpc_request_from_call() {
        let rpc_method = JsonRpcMethod::<NoParams, (), DefaultError>::new("test.method");
        let json_rpc_id = generate_random_rpc_id();
        let rpc_request = rpc_method.create_request_no_params(json_rpc_id);

        assert_eq!(rpc_request.method, "test.method");
        assert_eq!(rpc_request.jsonrpc, "2.0");
        assert_eq!(rpc_request.params, NoParams);
    }

    #[test]
    fn parse_rpc_response_success_from_call() {
        let rpc_method = JsonRpcMethod::<NoParams, String, DefaultError>::new("test.return_string");

        let json_value = serde_json::json!({
            "jsonrpc" : "2.0",
            "result" : "result_data",
            "id" : "request_id"
        });

        let json_str = serde_json::to_string(&json_value).unwrap();

        let result = rpc_method.parse_json_response_str(&json_str).unwrap();

        match result {
            JsonRpcResponse::Error(_) => panic!("Deserialized a good response but got panic"),
            JsonRpcResponse::Ok(ok) => {
                assert_eq!(ok.jsonrpc, "2.0");
                assert_eq!(ok.id, "request_id");
                assert_eq!(ok.result, "result_data")
            }
        }
    }

    #[test]
    fn parse_rpc_response_failure_from_call() {
        let rpc_method = JsonRpcMethod::<NoParams, String, DefaultError>::new("test.return_string");

        let json_value = serde_json::json!({
            "jsonrpc" : "2.0",
            "error" : { "code" : -32700, "message" : "Failed to parse response"},
            "id" : "request_id"
        });

        let json_str = serde_json::to_string(&json_value).unwrap();

        let result = rpc_method.parse_json_response_str(&json_str).unwrap();

        match result {
            JsonRpcResponse::Error(err) => {
                assert_eq!(err.jsonrpc, "2.0");

                assert_eq!(err.error.code, -32700);

                assert_eq!(err.error.message, "Failed to parse response");
                assert_eq!(err.id, JsonRpcId::String("request_id".to_string()));
            }
            JsonRpcResponse::Ok(_ok) => {
                panic!("Failure deserialized as Ok")
            }
        }
    }

    #[test]
    fn serialize_json_rpc_id() {
        let id_str = JsonRpcId::String("id_string".to_string());
        let id_i64 = JsonRpcId::Number(-12);
        let id_null = JsonRpcId::None;

        assert_eq!(serde_json::json!(id_str), serde_json::json!("id_string"));

        assert_eq!(serde_json::json!(id_i64), serde_json::json!(-12));

        assert_eq!(serde_json::json!(id_null), serde_json::json!(null))
    }

    #[test]
    fn deserialize_default_error() {
        let data = serde_json::json!({
            "jsonrpc" : "2.0",
            "id" : "abcdef",
            "error" : {
                "code" : 10,
                "message" : "Something happened",
                "data" : { "object" : "I am an unrecegonized object and should be either ignored or parsed" }
            }
        });

        let _: JsonRpcResponseFailure<DefaultError> = serde_json::from_value(data).unwrap();
    }
}
