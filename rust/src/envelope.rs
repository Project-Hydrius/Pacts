/*
 * Copyright © 2025 Hydrius, Project Hydrius, Wyrmlings
 * https://github.com/Project-Hydrius
 *
 * All rights reserved.
 *
 * This source code is part of the organizations named above.
 * Licensed for private use only. Unauthorized copying, modification,
 * or distribution is strictly prohibited.
 */

use crate::header::Header;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::Header;
    use serde_json::json;

    #[test]
    fn test_new_envelope() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({"id": "123", "name": "Test Item"});

        let envelope = Envelope::new(header.clone(), data.clone());

        assert_eq!(envelope.header(), &header);
        assert_eq!(envelope.data(), &data);
        assert!(envelope.metadata().is_none());
    }

    #[test]
    fn test_with_metadata() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({"id": "123"});
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!("test"));
        metadata.insert("priority".to_string(), json!("high"));

        let envelope = Envelope::with_metadata(header.clone(), data.clone(), metadata.clone());

        assert_eq!(envelope.header(), &header);
        assert_eq!(envelope.data(), &data);
        assert_eq!(envelope.metadata(), Some(&metadata));
    }

    #[test]
    fn test_with_primitive_data() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!("simple string data");

        let envelope = Envelope::new(header.clone(), data.clone());

        assert_eq!(envelope.header(), &header);
        assert_eq!(envelope.data(), &data);
    }

    #[test]
    fn test_with_null_data() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!(null);

        let envelope = Envelope::new(header.clone(), data.clone());

        assert_eq!(envelope.header(), &header);
        assert_eq!(envelope.data(), &data);
    }

    #[test]
    fn test_with_empty_metadata() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({"id": "123"});
        let empty_metadata = HashMap::new();

        let envelope =
            Envelope::with_metadata(header.clone(), data.clone(), empty_metadata.clone());

        assert_eq!(envelope.header(), &header);
        assert_eq!(envelope.data(), &data);
        assert_eq!(envelope.metadata(), Some(&empty_metadata));
    }

    #[test]
    fn test_with_complex_data() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({
            "user": {
                "id": "123",
                "profile": {
                    "name": "John Doe",
                    "age": 30,
                    "active": true
                }
            },
            "items": ["item1", "item2", "item3"]
        });

        let envelope = Envelope::new(header.clone(), data.clone());

        assert_eq!(envelope.header(), &header);
        assert_eq!(envelope.data(), &data);
    }

    #[test]
    fn test_serialization() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({"id": "123"});
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!("test"));

        let envelope = Envelope::with_metadata(header, data, metadata);

        let serialized = serde_json::to_string(&envelope).unwrap();
        let deserialized: Envelope = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            envelope.header.schema_version,
            deserialized.header.schema_version
        );
        assert_eq!(envelope.header.schema_id, deserialized.header.schema_id);
        assert_eq!(envelope.data, deserialized.data);
        assert_eq!(envelope.metadata, deserialized.metadata);
    }

    #[test]
    fn test_clone() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({"id": "123"});
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!("test"));

        let envelope = Envelope::with_metadata(header, data, metadata);
        let cloned = envelope.clone();

        assert_eq!(envelope.header.schema_version, cloned.header.schema_version);
        assert_eq!(envelope.header.schema_id, cloned.header.schema_id);
        assert_eq!(envelope.data, cloned.data);
        assert_eq!(envelope.metadata, cloned.metadata);
    }

    #[test]
    fn test_debug_format() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let data = json!({"id": "123"});

        let envelope = Envelope::new(header, data);
        let debug_str = format!("{:?}", envelope);

        assert!(debug_str.contains("Envelope"));
        assert!(debug_str.contains("test-schema"));
    }
}
