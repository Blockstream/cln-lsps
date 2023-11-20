use base64::Engine as _;
use serde::de::{DeserializeOwned, Deserializer, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use crate::error::map_json_rpc_error_code_to_str;
use std::str::FromStr;

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
/// but should also implement MapErrorCode.
///
/// The MapErrorCode trait is used to map error-codes
/// to human readable strings.
///
/// By default the values in the jsonrpc 2.0-specification
/// are used. However, it is possible to define additional
/// error-codes for a method.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcMethod<I, O, E>
where
    E: MapErrorCode,
{
    pub method: &'static str,
    #[serde(skip_serializing)]
    request: std::marker::PhantomData<I>,
    #[serde(skip_serializing)]
    return_type: std::marker::PhantomData<O>,
    #[serde(skip_serializing)]
    error_type: std::marker::PhantomData<E>,
}

impl<I, O, E> JsonRpcMethod<I, O, E>
where
    E: MapErrorCode,
{
    pub const fn new(method: &'static str) -> Self {
        Self {
            method,
            request: std::marker::PhantomData,
            return_type: std::marker::PhantomData,
            error_type: std::marker::PhantomData,
        }
    }

    pub const fn name(&self) -> &'static str {
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

    pub fn create_ok_response(request: JsonRpcRequest<I>, result: O) -> JsonRpcResponse<O, E> {
        JsonRpcResponse::Ok(JsonRpcResponseSuccess {
            jsonrpc: String::from("2.0"),
            id: request.id.clone(),
            result,
        })
    }

    pub fn into_typed_request(
        &self,
        request: JsonRpcRequest<serde_json::Value>,
    ) -> Result<JsonRpcRequest<I>, serde_json::Error>
    where
        I: DeserializeOwned,
    {
        let request = JsonRpcRequest::<I> {
            id: request.id,
            jsonrpc: request.jsonrpc,
            method: request.method,
            params: serde_json::from_value(request.params)?,
        };

        Ok(request)
    }
}

impl<O, E> JsonRpcMethod<NoParams, O, E>
where
    E: MapErrorCode,
{
    pub fn create_request_no_params(&self, json_rpc_id: JsonRpcId) -> JsonRpcRequest<NoParams> {
        self.create_request(NoParams::default(), json_rpc_id)
    }
}

impl<I, O, E> std::convert::From<&JsonRpcMethod<I, O, E>> for String
where
    E: MapErrorCode,
{
    fn from(value: &JsonRpcMethod<I, O, E>) -> Self {
        value.method.into()
    }
}

impl<'de, I, O, E> JsonRpcMethod<I, O, E>
where
    O: Deserialize<'de>,
    E: Deserialize<'de> + MapErrorCode,
{
    pub fn parse_json_response_str(
        &self,
        json_str: &'de str,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

impl<I, O, E> JsonRpcMethod<I, O, E>
where
    O: DeserializeOwned,
    E: DeserializeOwned + MapErrorCode,
{
    pub fn parse_json_response_value(
        &self,
        json_value: serde_json::Value,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error> {
        serde_json::from_value(json_value)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest<I> {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub method: String,
    pub params: I,
}

// LSPS0 specifies that the RPC-request must use a parameter-by-name structure.
//
// A JSONRpcRequest<(),()> will be serialized to a json where "params" : null
// A JsonRpcRequest<NoParams, ()> will be serialized to "params" : {} which is compliant
#[derive(Debug, Default, Clone, PartialEq)]
pub struct NoParams;

// Serde serializes NoParams to null by default
// LSPS0 requires an empty dictionary in this situation
impl Serialize for NoParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

impl<'de> Visitor<'de> for NoParams {
    type Value = NoParams;

    fn expecting(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Expected NoParams to be serialized as '{{}}'")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let maybe_entry = map.next_entry::<String, String>()?;
        match maybe_entry {
            Some(_) => Err(serde::de::Error::custom("Expected NoParams")),
            None => Ok(NoParams),
        }
    }
}

impl<'de> Deserialize<'de> for NoParams {
    fn deserialize<D>(deserializer: D) -> Result<NoParams, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(NoParams)
    }
}

impl<I> JsonRpcRequest<I> {
    pub fn new<O, E>(method: JsonRpcMethod<I, O, E>, params: I) -> Self
    where
        E: MapErrorCode,
    {
        return Self {
            jsonrpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: method.method.into(),
            params,
        };
    }
}

impl JsonRpcRequest<serde_json::Value> {
    pub fn deserialize<'de, I>(self) -> Result<JsonRpcRequest<I>, serde_json::Error>
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
    pub fn new_no_params<O, E>(method: JsonRpcMethod<NoParams, O, E>) -> Self
    where
        E: MapErrorCode,
    {
        return Self {
            jsonrpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: method.method.into(),
            params: NoParams::default(),
        };
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
pub struct ErrorData<E> {
    pub code: i64,
    pub message: String,
    pub data: Option<E>,
}

impl<E> ErrorData<E>
where
    E: MapErrorCode,
{
    pub fn code_str(&self) -> &str {
        return E::get_code_str(self.code);
    }
}

impl<E> ErrorData<E>
where
    E: MapErrorCode,
{
    pub fn into_response<O>(self, id: JsonRpcId) -> JsonRpcResponseFailure<E> {
        JsonRpcResponseFailure {
            id,
            jsonrpc: String::from("2.0"),
            error: self,
        }
    }
}

impl ErrorData<DefaultError> {
    pub fn parse_error(message: String) -> Self {
        Self {
            code: -32700,
            message,
            data: None,
        }
    }

    pub fn invalid_request(message: String) -> Self {
        Self {
            code: -32600,
            message,
            data: None,
        }
    }

    pub fn unknown_method(message: String) -> Self {
        Self {
            code: -32601,
            message,
            data: None,
        }
    }

    pub fn invalid_params(message: String) -> Self {
        Self {
            code: -32602,
            message,
            data: None,
        }
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultError(serde_json::Value);

pub trait MapErrorCode {
    fn get_code_str(code: i64) -> &'static str;
}

impl MapErrorCode for DefaultError {
    fn get_code_str(code: i64) -> &'static str {
        map_json_rpc_error_code_to_str(code)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn serialize_no_params() {
        let no_params = NoParams::default();
        let json_str = serde_json::to_string(&no_params).unwrap();

        assert_eq!(json_str, "{}")
    }

    #[test]
    fn deserialize_no_params() {
        let _: NoParams = serde_json::from_str("{}").unwrap();
    }

    #[test]
    fn serialize_json_rpc_request() {
        let rpc_request = JsonRpcRequest {
            id: "abcefg".into(),
            jsonrpc: "2.0".into(),
            params: NoParams::default(),
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
        assert_eq!(rpc_request.params, NoParams::default());
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
                assert_eq!(err.error.code_str(), "parsing_error");

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
