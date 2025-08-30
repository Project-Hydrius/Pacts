use crate::model::header::Header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Envelope struct that wraps data with metadata for schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    #[serde(rename = "header")]
    pub header: Header,

    #[serde(rename = "data")]
    pub data: serde_json::Value,

    #[serde(rename = "metadata")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Envelope {
    /// Creates a new envelope with header and data
    pub fn new(header: Header, data: serde_json::Value) -> Self {
        Self {
            header,
            data,
            metadata: None,
        }
    }

    /// Creates a new envelope with header, data, and metadata
    pub fn with_metadata(
        header: Header,
        data: serde_json::Value,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            header,
            data,
            metadata: Some(metadata),
        }
    }

    /// Gets the header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Gets the data
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }

    /// Gets the metadata
    pub fn metadata(&self) -> Option<&HashMap<String, serde_json::Value>> {
        self.metadata.as_ref()
    }
}