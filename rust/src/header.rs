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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Header struct that contains metadata about the envelope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Header {
    #[serde(rename = "schema_version")]
    pub schema_version: String,

    #[serde(rename = "schema_id")]
    pub schema_id: String,

    #[serde(rename = "timestamp")]
    pub timestamp: DateTime<Utc>,

    #[serde(rename = "content_type")]
    pub content_type: Option<String>,

    #[serde(rename = "auth_token")]
    pub auth_token: Option<String>,
}

impl Header {
    /// Creates a new header with schema version and ID
    pub fn new(schema_version: String, schema_id: String) -> Self {
        Self {
            schema_version,
            schema_id,
            timestamp: Utc::now(),
            content_type: None,
            auth_token: None,
        }
    }

    /// Creates a new header with schema version, ID, and content type
    pub fn with_content_type(
        schema_version: String,
        schema_id: String,
        content_type: String,
    ) -> Self {
        Self {
            schema_version,
            schema_id,
            timestamp: Utc::now(),
            content_type: Some(content_type),
            auth_token: None,
        }
    }

    /// Creates a new header with schema version, ID, content type, and auth token
    pub fn with_auth(
        schema_version: String,
        schema_id: String,
        content_type: Option<String>,
        auth_token: String,
    ) -> Self {
        Self {
            schema_version,
            schema_id,
            timestamp: Utc::now(),
            content_type,
            auth_token: Some(auth_token),
        }
    }

    /// Gets the schema version
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Gets the schema ID
    pub fn schema_id(&self) -> &str {
        &self.schema_id
    }

    /// Gets the timestamp
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Gets the content type
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Gets the auth token
    pub fn auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_new_header() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());

        assert_eq!(header.schema_version(), "1.0");
        assert_eq!(header.schema_id(), "test-schema");
        assert!(header.content_type().is_none());

        // Verify timestamp is recent (within last 5 seconds)
        let now = Utc::now();
        assert!(header.timestamp() > &(now - Duration::seconds(5)));
        assert!(header.timestamp() < &(now + Duration::seconds(5)));
    }

    #[test]
    fn test_with_content_type() {
        let header = Header::with_content_type(
            "2.0".to_string(),
            "test-schema".to_string(),
            "application/json".to_string(),
        );

        assert_eq!(header.schema_version(), "2.0");
        assert_eq!(header.schema_id(), "test-schema");
        assert_eq!(header.content_type(), Some("application/json"));

        // Verify timestamp is recent
        let now = Utc::now();
        assert!(header.timestamp() > &(now - Duration::seconds(5)));
        assert!(header.timestamp() < &(now + Duration::seconds(5)));
    }

    #[test]
    fn test_empty_strings() {
        let header = Header::new("".to_string(), "".to_string());

        assert_eq!(header.schema_version(), "");
        assert_eq!(header.schema_id(), "");
        assert!(header.content_type().is_none());
    }

    #[test]
    fn test_special_characters() {
        let header = Header::with_content_type(
            "2.0-beta".to_string(),
            "schema-with-special-chars_123".to_string(),
            "application/xml; charset=utf-8".to_string(),
        );

        assert_eq!(header.schema_version(), "2.0-beta");
        assert_eq!(header.schema_id(), "schema-with-special-chars_123");
        assert_eq!(
            header.content_type(),
            Some("application/xml; charset=utf-8")
        );
    }

    #[test]
    fn test_timestamp_consistency() {
        let header1 = Header::new("1.0".to_string(), "test-schema".to_string());
        let header2 = Header::new("1.0".to_string(), "test-schema".to_string());

        // Both should have recent timestamps
        let now = Utc::now();
        assert!(header1.timestamp() > &(now - Duration::seconds(5)));
        assert!(header2.timestamp() > &(now - Duration::seconds(5)));

        // Timestamps should be different (created at different times)
        assert_ne!(header1.timestamp(), header2.timestamp());
    }

    #[test]
    fn test_serialization() {
        let header = Header::with_content_type(
            "1.0".to_string(),
            "test-schema".to_string(),
            "application/json".to_string(),
        );

        let serialized = serde_json::to_string(&header).unwrap();
        let deserialized: Header = serde_json::from_str(&serialized).unwrap();

        assert_eq!(header.schema_version, deserialized.schema_version);
        assert_eq!(header.schema_id, deserialized.schema_id);
        assert_eq!(header.content_type, deserialized.content_type);
        // Timestamps might be slightly different due to serialization precision
        assert!(
            header
                .timestamp()
                .signed_duration_since(deserialized.timestamp())
                .abs()
                < Duration::milliseconds(1)
        );
    }

    #[test]
    fn test_deserialization_without_content_type() {
        let json = r#"{
            "schema_version": "1.0",
            "schema_id": "test-schema",
            "timestamp": "2023-01-01T00:00:00Z"
        }"#;

        let header: Header = serde_json::from_str(json).unwrap();

        assert_eq!(header.schema_version(), "1.0");
        assert_eq!(header.schema_id(), "test-schema");
        assert!(header.content_type().is_none());
    }

    #[test]
    fn test_clone() {
        let header = Header::with_content_type(
            "1.0".to_string(),
            "test-schema".to_string(),
            "application/json".to_string(),
        );

        let cloned = header.clone();

        assert_eq!(header.schema_version, cloned.schema_version);
        assert_eq!(header.schema_id, cloned.schema_id);
        assert_eq!(header.content_type, cloned.content_type);
        assert_eq!(header.timestamp, cloned.timestamp);
    }

    #[test]
    fn test_debug_format() {
        let header = Header::new("1.0".to_string(), "test-schema".to_string());
        let debug_str = format!("{:?}", header);

        assert!(debug_str.contains("Header"));
        assert!(debug_str.contains("1.0"));
        assert!(debug_str.contains("test-schema"));
    }

    #[test]
    fn test_getter_methods() {
        let header = Header::with_content_type(
            "3.0".to_string(),
            "another-schema".to_string(),
            "text/plain".to_string(),
        );

        assert_eq!(header.schema_version(), "3.0");
        assert_eq!(header.schema_id(), "another-schema");
        assert_eq!(header.content_type(), Some("text/plain"));
        assert!(header.timestamp() <= &Utc::now());
    }
}
