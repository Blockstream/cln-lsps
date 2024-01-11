use core::str::FromStr;
use std::fmt::{Display, Formatter};

use anyhow::{anyhow, Context, Result};
use bitcoin::address::Address;
pub use bitcoin::address::{NetworkChecked, NetworkUnchecked, NetworkValidation};
pub use bitcoin::network::Network;

use serde::de::Error as SeError;
use serde::ser::Error as DeError;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use time::format_description::FormatItem;
use time::macros::{format_description, offset};
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::secp256k1::PublicKey as _PublicKey;

#[derive(Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct OnchainAddress {
    pub address: Address,
}

impl<'de> Deserialize<'de> for OnchainAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let address_unchecked = Address::<NetworkUnchecked>::deserialize(deserializer)?;
        Ok(OnchainAddress {
            address: address_unchecked.assume_checked(),
        })
    }
}

pub trait NetworkCheckable {
    fn require_network(&self, network: &bitcoin::Network) -> Result<()>;
}

impl NetworkCheckable for OnchainAddress {
    fn require_network(&self, network: &bitcoin::Network) -> Result<()> {
        if network != self.address.network() {
            Err(anyhow!(
                "Network mismatch: Expected {} but got {}",
                network,
                self.address.network()
            ))
        } else {
            Ok(())
        }
    }
}

impl NetworkCheckable for Option<OnchainAddress> {
    fn require_network(&self, network: &bitcoin::Network) -> Result<()> {
        match self {
            Some(x) => x.require_network(network),
            None => Ok(()),
        }
    }
}

impl FromStr for OnchainAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = Address::<NetworkUnchecked>::from_str(s)?;
        Ok(Self {
            address: address.assume_checked(),
        })
    }
}

impl Display for OnchainAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicKey(_PublicKey);

impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let x = self.inner().serialize();
        x.hash(state);
    }
}

impl From<_PublicKey> for PublicKey {
    fn from(public_key: _PublicKey) -> Self {
        Self(public_key)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = hex::encode(self.0.serialize());
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PublicKeyVisitor;

        impl Visitor<'_> for PublicKeyVisitor {
            type Value = PublicKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "A compressed public-key that is hex-encoded")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let data =
                    hex::decode(v).map_err(|_| serde::de::Error::custom("Expected valid hex"))?;
                let pubkey = _PublicKey::from_slice(&data).map_err(|err| {
                    serde::de::Error::custom(format!("Invalid public-key: {}", err))
                })?;
                Ok(pubkey.into())
            }
        }

        let visitor = PublicKeyVisitor;
        deserializer.deserialize_str(visitor)
    }
}

impl PublicKey {
    pub fn from_hex(hex: &str) -> Result<Self> {
        let data = hex::decode(hex).context("Invalid hex")?;
        let publickey =
            _PublicKey::from_slice(&data).map_err(|m| anyhow!("Error parsing PublicKey: {}", m))?;
        Ok(PublicKey(publickey))
    }

    pub fn to_hex(&self) -> String {
        let data = self.0.serialize();
        hex::encode(data)
    }

    pub fn inner(self) -> _PublicKey {
        self.0
    }
}

// Implements all the common schema's defined in LSPS0 common schema's

// Initially I used serde_as for the parsing and serialization of this type.
// However, the spec is more strict.
// It requires a yyyy-mm-ddThh:mm:ss.uuuZ format
//
// The serde_as provides us options such as rfc_3339.
// Note, that this also allows formats that are not compliant to the LSP-spec such as dropping
// the fractional seconds or use non UTC timezones.
//
// For LSPS2 the `valid_until`-field must be copied verbatim. As a client this can only be
// achieved if the LSPS2 sends a fully compliant timestamp.
//
// I have decided to fail early if another timestamp is received
#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub struct IsoDatetime {
    pub datetime: PrimitiveDateTime,
}

const DATETIME_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

impl IsoDatetime {
    pub fn now() -> Self {
        Self::from_offset_date_time(time::OffsetDateTime::now_utc())
    }

    pub fn from_offset_date_time(datetime: OffsetDateTime) -> Self {
        let offset = time::UtcOffset::from_whole_seconds(0).unwrap();
        let datetime_utc = datetime.to_offset(offset);
        let primitive = PrimitiveDateTime::new(datetime_utc.date(), datetime.time());
        Self {
            datetime: primitive,
        }
    }

    pub fn from_unix_timestamp(value: i64) -> Result<Self> {
        let offset =
            OffsetDateTime::from_unix_timestamp(value).context("Failed to construct datetime")?;
        Ok(Self::from_offset_date_time(offset))
    }

    pub fn unix_timestamp(&self) -> i64 {
        self.datetime.assume_offset(offset!(UTC)).unix_timestamp()
    }

    pub fn from_primitive_date_time(datetime: PrimitiveDateTime) -> Self {
        Self { datetime }
    }

    pub fn datetime(&self) -> OffsetDateTime {
        self.datetime.assume_utc()
    }
}

impl Serialize for IsoDatetime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let datetime_str = self
            .datetime
            .format(&DATETIME_FORMAT)
            .map_err(|err| S::Error::custom(format!("Failed to format datetime {:?}", err)))?;

        serializer.serialize_str(&datetime_str)
    }
}

impl<'de> Deserialize<'de> for IsoDatetime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        time::PrimitiveDateTime::parse(&str_repr, DATETIME_FORMAT)
            .map_err(|err| D::Error::custom(format!("Failed to parse Datetime. {:?}", err)))
            .map(Self::from_primitive_date_time)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Copy, Default)]
pub struct SatAmount(u64);
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Copy, Default)]
pub struct MsatAmount(u64);

impl std::fmt::Display for SatAmount {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{} sat", self.0)
    }
}

impl std::fmt::Display for MsatAmount {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{} msat", self.0)
    }
}

impl SatAmount {
    pub fn sat_value(&self) -> u64 {
        self.0
    }

    pub fn new(value: u64) -> Self {
        SatAmount(value)
    }
}

impl MsatAmount {
    pub fn msat_value(&self) -> u64 {
        self.0
    }

    pub fn new(value: u64) -> Self {
        MsatAmount(value)
    }
}

impl Serialize for SatAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let amount_str = self.0.to_string();
        serializer.serialize_str(&amount_str)
    }
}

impl Serialize for MsatAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let amount_str = self.0.to_string();
        serializer.serialize_str(&amount_str)
    }
}

impl<'de> Deserialize<'de> for SatAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        let u64_repr: Result<u64, _> = str_repr
            .parse()
            .map_err(|_| D::Error::custom(String::from("Failed to parse sat_amount")));
        Ok(Self(u64_repr.unwrap()))
    }
}

impl<'de> Deserialize<'de> for MsatAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        let u64_repr: Result<u64, _> = str_repr
            .parse()
            .map_err(|_| D::Error::custom(String::from("Failed to parse sat_amount")));
        Ok(Self(u64_repr.unwrap()))
    }
}

impl SatAmount {
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let sat_value = self.0.checked_add(other.0)?;
        Some(SatAmount::new(sat_value))
    }
}

impl MsatAmount {
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let sat_value = self.0.checked_add(other.0)?;
        Some(MsatAmount::new(sat_value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortChannelId(u64);

impl Serialize for ShortChannelId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ShortChannelId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|e| Error::custom(e.to_string()))
    }
}

impl FromStr for ShortChannelId {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Result<Vec<u64>, _> = s.split('x').map(|p| p.parse()).collect();
        let parts = parts.with_context(|| format!("Malformed short_channel_id: {}", s))?;
        if parts.len() != 3 {
            return Err(anyhow!(
                "Malformed short_channel_id: element count mismatch"
            ));
        }

        Ok(ShortChannelId(
            (parts[0] << 40) | (parts[1] << 16) | parts[2],
        ))
    }
}
impl Display for ShortChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}x{}", self.block(), self.txindex(), self.outnum())
    }
}
impl ShortChannelId {
    pub fn block(&self) -> u32 {
        (self.0 >> 40) as u32 & 0xFFFFFF
    }
    pub fn txindex(&self) -> u32 {
        (self.0 >> 16) as u32 & 0xFFFFFF
    }
    pub fn outnum(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeeRate {
    fee_rate: u64,
}

impl FeeRate {
    pub fn from_sats_per_kwu(sats_kwu: u64) -> Self {
        Self { fee_rate: sats_kwu }
    }

    pub fn to_sats_per_kwu(&self) -> u64 {
        self.fee_rate
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionId([u8; 32]);

impl TransactionId {
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        let txid: [u8; 32] = data.try_into().context("Invalid transaction Id")?;
        Ok(Self(txid))
    }
}

impl Serialize for TransactionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = self.to_string();
        serializer.serialize_str(&str_value)
    }
}

impl<'de> Deserialize<'de> for TransactionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionIdVisitor;

        impl<'de> Visitor<'de> for TransactionIdVisitor {
            type Value = TransactionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "Valid TransactionId")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SeError,
            {
                Self::Value::from_str(v).map_err(|e| E::custom(e.to_string()))
            }
        }

        let visitor = TransactionIdVisitor;
        deserializer.deserialize_str(visitor)
    }
}

impl FromStr for TransactionId {
    type Err = anyhow::Error;

    fn from_str(txid: &str) -> Result<Self> {
        let txid = hex::decode(txid).context("txid is not valid hex")?;
        Self::from_slice(&txid)
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txid_hex = hex::encode(self.0);
        write!(f, "{}", txid_hex)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outpoint {
    txid: TransactionId,
    outnum: u32,
}

impl Display for Outpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.txid, self.outnum)
    }
}

impl FromStr for Outpoint {
    type Err = anyhow::Error;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = v.split(":").collect();

        if split.len() != 2 {
            return Err(anyhow!("Invalid outpoint. Should be a txid and outnum. e.g: '4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0'"));
        }

        let txid = split[0];
        let outnum = split[1];

        let txid = TransactionId::from_str(txid).context("Invalid outpoint (txid)")?;
        let outnum = u32::from_str(outnum).context("Invalid outpoint (outnum)")?;

        Ok(Self { txid, outnum })
    }
}

impl Serialize for Outpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let outnum_str = self.to_string();
        serializer.serialize_str(&outnum_str)
    }
}

impl<'de> Deserialize<'de> for Outpoint {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OutnumVisitor;

        impl<'de> Visitor<'de> for OutnumVisitor {
            type Value = Outpoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "A valid outpoint")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SeError,
            {
                Self::Value::from_str(v).map_err(|e| E::custom(e.to_string()))
            }
        }

        let visitor = OutnumVisitor;
        deserializer.deserialize_str(visitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_amount_sats() {
        // Pick a number which exceeds 2^32 to ensure internal representation exceeds 32 bits
        let json_str_number = "\"10000000001\"";

        let int_number: u64 = 10000000001;

        let x = serde_json::from_str::<SatAmount>(json_str_number).unwrap();
        assert_eq!(x.sat_value(), int_number);
    }

    #[test]
    fn serializing_amount_sats() {
        // Pick a number which exceeds 2^32 to ensure internal representation exceeds 32 bits
        // The json_str includes the " to indicate it is a string
        let json_str_number = "\"10000000001\"";
        let int_number: u64 = 10000000001;

        let sat_amount = SatAmount::new(int_number);

        let json_str = serde_json::to_string::<SatAmount>(&sat_amount).unwrap();
        assert_eq!(json_str, json_str_number);
    }

    #[test]
    fn parse_and_serialize_datetime() {
        let datetime_str = "\"2023-01-01T23:59:59.999Z\"";

        let dt = serde_json::from_str::<IsoDatetime>(datetime_str).unwrap();

        assert_eq!(dt.datetime.year(), 2023);
        assert_eq!(dt.datetime.month(), time::Month::January);
        assert_eq!(dt.datetime.day(), 1);
        assert_eq!(dt.datetime.hour(), 23);
        assert_eq!(dt.datetime.minute(), 59);
        assert_eq!(dt.datetime.second(), 59);

        assert_eq!(
            serde_json::to_string(&dt).expect("Can be serialized"),
            datetime_str
        )
    }

    #[test]
    fn parse_datetime_that_doesnt_follow_spec() {
        // The spec doesn't explicitly say that clients have to ignore datetimes that don't follow the spec
        // However, in LSPS2 the datetime_str must be repeated verbatim
        let datetime_str = "\"2023-01-01T23:59:59.99Z\"";

        let result = serde_json::from_str::<IsoDatetime>(datetime_str);
        result.expect_err("datetime_str should not be parsed if it doesn't follow spec");
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn parse_scid_from_string() {
        // How to read this test
        //
        // The shortchannel_id is 8 bytes long.
        // The 3 first bytes are the blockheight, 3 next bytes are the txid and last 2 bytes are vout
        // This explains the unusual byte groupings

        // The string representation are the same numbers separated by the letter x

        // Test the largest possible value
        let scid_str = "16777215x16777215x65535";

        let scid = ShortChannelId::from_str(scid_str).expect("The scid is parseable");
        assert_eq!(scid.to_string(), scid_str);

        // Test the smallest possible value
        let scid_str = "0x0x0";

        let scid = ShortChannelId::from_str(scid_str).expect("The scid is parseable");
        assert_eq!(scid.to_string(), scid_str);

        let scid_str = "1x2x3";

        let scid = ShortChannelId::from_str(scid_str).expect("The scid is parseable");
        assert_eq!(scid.to_string(), scid_str);
        // A couple of unparseable scids
        assert!(ShortChannelId::from_str("xx").is_err());
        assert!(ShortChannelId::from_str("0x0").is_err());
        assert!(ShortChannelId::from_str("-2x-12x14").is_err());
    }

    #[test]
    fn short_channel_id_is_serialized_as_str() {
        let scid: ShortChannelId = ShortChannelId::from_str("10x5x8").unwrap();
        let scid_json_obj = serde_json::to_string(&scid).expect("Can be serialized");
        assert_eq!("\"10x5x8\"", scid_json_obj);
    }

    #[test]
    fn short_channel_id_can_be_deserialized_from_str() {
        let scid_json = "\"11x12x13\"";

        let scid = serde_json::from_str::<ShortChannelId>(scid_json).expect("scid can be parsed");

        assert_eq!(scid, ShortChannelId::from_str("11x12x13").unwrap());
    }

    use secp256k1::Secp256k1;
    use secp256k1::SecretKey;

    #[test]
    fn pubkey_is_serialized_as_hex() {
        // Generate a public and private keypair
        let mut rng = rand::thread_rng();
        let s = Secp256k1::signing_only();
        let secret_key = SecretKey::new(&mut rng);

        let public_key = _PublicKey::from_secret_key(&s, &secret_key);
        let pub_key_hex = hex::encode(public_key.serialize());

        // Convert the to our schema object
        let public_key = PublicKey::from(public_key);

        // Serialize the string to a json_value
        let json_value = serde_json::json!(public_key);

        // Compute the compressed hex-encoded string that represents the public key

        assert_eq!(
            pub_key_hex, json_value,
            "The key should be serialized as a compressed hex"
        );
    }

    #[test]
    fn deserialize_onchain_address() {
        let regtest_address_json = "\"bcrt1qkm08480v79rzjp7tx2pjrly423ncv85k65nsmu\"";
        let parsed = serde_json::from_str::<OnchainAddress>(regtest_address_json).unwrap();
        parsed
            .clone()
            .require_network(&Network::Regtest)
            .expect("Is okay because it is a regtest address");
        parsed
            .clone()
            .require_network(&Network::Bitcoin)
            .expect_err("Should fail because the address is not a mainnet address");

        // We don't te test the NetworkChecked case here
        // Deserializing into a NetworkChecked type

        // This is a compiler error
        //
        // let parsed = serde_json::from_str::<OnchainAddress<NetworkChecked>>(regtest_address_json).unwrap();
    }

    #[test]
    fn serialize_onchain_address() {
        // Generate random key pair.
        let address_unchecked: Address<NetworkUnchecked> =
            "32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf".parse().unwrap();

        // NetworkUnchecked cannot be serialized
        let json_value = serde_json::to_value(address_unchecked.clone()).unwrap();
        assert_eq!(json_value, "32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf");

        // NetworkChecked can be serialized
        let address_checked = address_unchecked.require_network(Network::Bitcoin).unwrap();
        let json_value = serde_json::to_value(address_checked).unwrap();
        assert_eq!(json_value, "32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf");
    }

    #[test]
    fn serialize_fee_rate() {
        let min_fee = FeeRate::from_sats_per_kwu(253);
        let json_value = serde_json::to_value(min_fee).unwrap();

        assert_eq!(json_value, serde_json::json!(253));
    }

    #[test]
    fn serialize_txid_type() {
        let txid_str = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
        let tx = TransactionId::from_str(txid_str).unwrap();
        assert_eq!(tx.to_string(), txid_str);

        let tx_deserialized: TransactionId =
            serde_json::from_value(serde_json::json!(txid_str)).unwrap();
        assert_eq!(
            tx, tx_deserialized,
            "Failed to deserialize transaction correctly"
        );

        let tx_serialized = serde_json::to_value(&tx).unwrap();
        assert_eq!(tx_serialized, serde_json::json!(txid_str));
    }

    #[test]
    fn serialize_outpoint() {
        let outpoint_str = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0";
        let outpoint = Outpoint::from_str(outpoint_str).unwrap();
        assert_eq!(outpoint.to_string(), outpoint_str);

        let outpoint_deserialized: Outpoint =
            serde_json::from_value(serde_json::json!(outpoint_str)).unwrap();
        assert_eq!(
            outpoint, outpoint_deserialized,
            "Failed to deserialize transaction correctly"
        );

        let outpoint_serialized = serde_json::to_value(&outpoint).unwrap();
        assert_eq!(outpoint_serialized, serde_json::json!(outpoint_str));
    }
}
