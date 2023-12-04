use anyhow::{Context, Result};
use lsp_primitives::lsps0::common_schemas::{IsoDatetime, MsatAmount, SatAmount};

pub trait IntoSqliteInteger {
    fn into_sqlite_integer(&self) -> Result<i64>;
}

pub trait FromSqliteInteger
where
    Self: Sized,
{
    fn from_sqlite_integer(value: i64) -> Result<Self>;
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
