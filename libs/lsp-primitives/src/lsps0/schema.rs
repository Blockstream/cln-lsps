pub use crate::lsps0::common_schemas::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListprotocolsResponse {
    pub protocols: Vec<u32>,
}

impl ListprotocolsResponse {
    pub fn new(protocols: Vec<u32>) -> Self {
        Self { protocols }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize_protocol_list() {
        let protocols = ListprotocolsResponse {
            protocols: vec![1, 3],
        };

        let json_str = serde_json::to_string(&protocols).unwrap();
        assert_eq!(json_str, "{\"protocols\":[1,3]}")
    }
}
