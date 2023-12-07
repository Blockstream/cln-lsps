/// Defines all RPC-methods
///
///
///
use crate::json_rpc::{DefaultError, JsonRpcMethod, NoParams};
pub use crate::lsps0::schema::ListprotocolsResponse;
pub use crate::lsps1::schema::{
    Lsps1CreateOrderRequest, Lsps1CreateOrderResponse, Lsps1GetOrderRequest, Lsps1GetOrderResponse,
    Lsps1InfoRequest, Lsps1InfoResponse,
};
pub use crate::lsps2::schema::{
    Lsps2BuyError, Lsps2BuyRequest, Lsps2BuyResponse, Lsps2GetInfoError, Lsps2GetInfoRequest,
    Lsps2GetInfoResponse, Lsps2GetVersionsResponse,
};

use serde::de::{Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Result};

// Every time I read this file I am tempted to write a macro that
// generates it. I've tried it a few times but the result never improved
// the code-base.
//
// If you can write a macro that makes this code more readable, shorter and
// easier to maintain let me know. I'd be happy to merge

// All rpc-methods defined in the LSPS standard
// The generics are <I,O,E> where
// - I represents the params
// - O represents the result data
// - E represents the error if present
//
// To create language bindings for a new rpc-call you must
// 1. Add it to the JsonRpcMethodEnum
// 2. Add it to the from_method_name function
// 3. Add it to the ref_erase function
pub type Lsps0ListProtocols = JsonRpcMethod<'static, NoParams, ListprotocolsResponse, DefaultError>;

pub type Lsps1Info = JsonRpcMethod<'static, Lsps1InfoRequest, Lsps1InfoResponse, DefaultError>;
pub type Lsps1CreateOrder =
    JsonRpcMethod<'static, Lsps1CreateOrderRequest, Lsps1CreateOrderResponse, DefaultError>;

pub type Lsps1GetOrder =
    JsonRpcMethod<'static, Lsps1GetOrderRequest, Lsps1GetOrderResponse, DefaultError>;

// LSPS0: Transport layer
pub const LSPS0_LIST_PROTOCOLS: Lsps0ListProtocols =
    Lsps0ListProtocols::new("lsps0.list_protocols");

// LSPS1: Buy Channels
pub const LSPS1_GETINFO: Lsps1Info = Lsps1Info::new("lsps1.info");
pub const LSPS1_CREATE_ORDER: Lsps1CreateOrder = Lsps1CreateOrder::new("lsps1.create_order");
pub const LSPS1_GET_ORDER: Lsps1GetOrder = Lsps1GetOrder::new("lsps1.get_order");

pub enum JsonRpcMethodEnum {
    Lsps0ListProtocols(Lsps0ListProtocols),
    Lsps1Info(Lsps1Info),
    Lsps1CreateOrder(Lsps1CreateOrder),
    Lsps1GetOrder(Lsps1GetOrder),
}

impl Serialize for JsonRpcMethodEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for JsonRpcMethodEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsonRpcMethodEnumVisitor;

        impl<'de> Visitor<'de> for JsonRpcMethodEnumVisitor {
            type Value = JsonRpcMethodEnum;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A valid rpc-method name")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let result = JsonRpcMethodEnum::from_method_name(value);
                result.map_err(|_| E::custom(format!("Unknown method '{}'", value)))
            }
        }

        deserializer.deserialize_str(JsonRpcMethodEnumVisitor)
    }
}

impl JsonRpcMethodEnum {
    pub fn from_method_name(value: &str) -> Result<JsonRpcMethodEnum> {
        match value {
            "lsps0.list_protocols" => Ok(Self::Lsps0ListProtocols(LSPS0_LIST_PROTOCOLS)),
            "lsps1.info" => Ok(Self::Lsps1Info(LSPS1_GETINFO)),
            "lsps1.create_order" => Ok(Self::Lsps1CreateOrder(LSPS1_CREATE_ORDER)),
            "lsps1.get_order" => Ok(Self::Lsps1GetOrder(LSPS1_GET_ORDER)),
            default => Err(anyhow!("Unknown method '{}'", default)),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Lsps0ListProtocols(x) => x.name(),
            Self::Lsps1Info(x) => x.name(),
            Self::Lsps1CreateOrder(x) => x.name(),
            Self::Lsps1GetOrder(x) => x.name(),
        }
    }
}
