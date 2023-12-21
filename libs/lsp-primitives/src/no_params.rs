use crate::lsps0::parameter_validation::ExpectedFields;
use serde::{
    de::{Deserializer, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

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
        S: Serializer,
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

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(NoParams)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(NoParams)
    }
}

impl<'de> Deserialize<'de> for NoParams {
    fn deserialize<D>(deserializer: D) -> Result<NoParams, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NoParams)
    }
}

impl ExpectedFields for NoParams {
    fn expected_fields() -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize_no_params() {
        let no_params = NoParams;
        let json_str = serde_json::to_string(&no_params).unwrap();

        assert_eq!(json_str, "{}")
    }

    #[test]
    fn deserialize_no_params() {
        let _: NoParams = serde_json::from_str("{}").unwrap();
        let _: NoParams = serde_json::from_str("null").unwrap();
        let _: NoParams = serde_json::from_value(serde_json::Value::Null).unwrap();
    }
}
