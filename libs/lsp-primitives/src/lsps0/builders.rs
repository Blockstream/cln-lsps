use anyhow::{Context, Result};

use crate::lsps0::schema::ListprotocolsResponse;

#[derive(Debug, Default)]
pub struct ListprotocolsResponseBuilder {
    protocols: Option<Vec<u32>>,
}

impl ListprotocolsResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn protocols(mut self, protocols: Vec<u32>) -> Self {
        self.protocols = Some(protocols);
        self
    }

    pub fn build(self) -> Result<ListprotocolsResponse> {
        let protocols = self.protocols.context("Missing field 'protocols'")?;

        let result = ListprotocolsResponse {
            protocols,
        };

        Ok(result)
    }
}
