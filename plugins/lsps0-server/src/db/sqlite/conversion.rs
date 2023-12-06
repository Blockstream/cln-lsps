use anyhow::{anyhow, Context, Result};
use lsp_primitives::lsps0::common_schemas::{FeeRate, IsoDatetime, MsatAmount, SatAmount};
use lsp_primitives::lsps1::schema::OrderState;

pub trait IntoSqliteInteger {
    fn into_sqlite_integer(&self) -> Result<i64>;
}

pub trait FromSqliteInteger
where
    Self: Sized,
{
    fn from_sqlite_integer(value: i64) -> Result<Self>;
}

impl IntoSqliteInteger for u8 {
    fn into_sqlite_integer(&self) -> Result<i64> {
        i64::try_from(*self).context(format!("Failed to fit {} in i64", self))
    }
}

impl FromSqliteInteger for u8 {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        u8::try_from(value).context(format!("Failed to fit {} in i64", value))
    }
}

impl IntoSqliteInteger for u64 {
    fn into_sqlite_integer(&self) -> Result<i64> {
        i64::try_from(*self).context(format!("Failed to fit {} in i64", self))
    }
}

impl FromSqliteInteger for u64 {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        u64::try_from(value).context(format!("Failed to fit {} in i64", value))
    }
}

impl IntoSqliteInteger for SatAmount {
    fn into_sqlite_integer(&self) -> Result<i64> {
        i64::try_from(self.sat_value()).context(format!("Failed fit {} in i64", self))
    }
}

impl FromSqliteInteger for SatAmount {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        let vu64 = u64::try_from(value).context("Failed to represent negative SatAmount")?;
        Ok(SatAmount::new(vu64))
    }
}

impl IntoSqliteInteger for MsatAmount {
    fn into_sqlite_integer(&self) -> Result<i64> {
        i64::try_from(self.msat_value()).context(format!("Failed fit {} in i64", self))
    }
}

impl FromSqliteInteger for MsatAmount {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        let vu64 = u64::try_from(value).context("Failed to represent negative SatAmount")?;
        Ok(MsatAmount::new(vu64))
    }
}

impl IntoSqliteInteger for IsoDatetime {
    fn into_sqlite_integer(&self) -> Result<i64> {
        Ok(self.datetime().unix_timestamp())
    }
}

impl FromSqliteInteger for IsoDatetime {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        IsoDatetime::from_unix_timestamp(value)
    }
}

impl IntoSqliteInteger for FeeRate {
    fn into_sqlite_integer(&self) -> Result<i64> {
        self.to_sats_per_kwu().into_sqlite_integer()
    }
}

impl FromSqliteInteger for FeeRate {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        let fee_rate = u64::from_sqlite_integer(value)?;
        Ok(FeeRate::from_sats_per_kwu(fee_rate))
    }
}

impl IntoSqliteInteger for OrderState {
    fn into_sqlite_integer(&self) -> Result<i64> {
        Ok(match self {
            OrderState::Created => 1,
            OrderState::Completed => 2,
            OrderState::Failed => 3,
        })
    }
}

impl FromSqliteInteger for OrderState {
    fn from_sqlite_integer(value: i64) -> Result<Self> {
        match value {
            1 => Ok(OrderState::Created),
            2 => Ok(OrderState::Completed),
            3 => Ok(OrderState::Failed),
            _ => Err(anyhow!("Unknown order state")),
        }
    }
}
