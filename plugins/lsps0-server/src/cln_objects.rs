use serde::de::{Deserializer, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct AmountMsat {
    pub(crate) msat: u64,
}

impl<'de> Deserialize<'de> for AmountMsat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MsatVisitor;

        impl<'de> Visitor<'de> for MsatVisitor {
            type Value = AmountMsat;

            fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(fmt, "An integer representing an amount in Msat")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !v.ends_with("msat") {
                    return Err(serde::de::Error::custom(
                        "If msat is a string it should end with msat",
                    ));
                }

                let v_amount = &v[..v.len() - 4];
                let v = v_amount
                    .parse::<u64>()
                    .map_err(|_| serde::de::Error::custom(format!("Invalid MsatAmount {}", v)))?;
                Ok(AmountMsat { msat: v })
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(AmountMsat { msat: v })
            }
        }

        deserializer.deserialize_any(MsatVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PaymentHook {
    pub(crate) payment: Payment,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Payment {
    pub(crate) label: String,
    pub(crate) preimage: String,
    pub(crate) msat: AmountMsat,
}

pub(crate) struct PaymentHookResponse;

impl PaymentHookResponse {
    pub(crate) fn r#continue() -> serde_json::Value {
        serde_json::json!({"result" : "continue"})
    }

    pub(crate) fn reject() -> serde_json::Value {
        serde_json::json!({"result" : "reject"})
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn deserialize_payment_hook() {
        let payment_hook = serde_json::json!(
        {"payment": {
            "label": "unique-label-for-invoice",
            "preimage": "0000000000000000000000000000000000000000000000000000000000000000",
            "msat": 1234
        }});

        let _: PaymentHook = serde_json::from_value(payment_hook).unwrap();
    }

    #[test]
    fn deserialize_amount_msat_as_u64() {
        // Some breaking changes occured in Core Lightning version 23.1
        // See 0b23133ab2
        //
        // The data passed in the PaymentHook changed.

        // New behavior.
        // The value is represented as a number
        let value = serde_json::json!(1234);
        let _: AmountMsat = serde_json::from_value(value).unwrap();

        // Old behavior
        // The value is represented as an msat string
        let msat_value = serde_json::json!("1234msat");
        let _: AmountMsat = serde_json::from_value(msat_value).unwrap();
    }
}
