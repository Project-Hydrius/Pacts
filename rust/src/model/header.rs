use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Header struct that contains metadata about the envelope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Header {
    #[serde(rename = "schema_version")]
    pub schema_version: String,

    #[serde(rename = "schema_category")]
    pub schema_category: String,

    #[serde(rename = "schema_name")]
    pub schema_name: String,

    #[serde(rename = "timestamp")]
    pub timestamp: DateTime<Utc>,

    #[serde(rename = "content_type")]
    pub content_type: Option<String>,
}

impl Header {
    /// Creates a new header with schema version, category, and name
    pub fn new(schema_version: String, schema_category: String, schema_name: String) -> Self {
        Self {
            schema_version,
            schema_category,
            schema_name,
            timestamp: Utc::now(),
            content_type: None,
        }
    }

    /// Creates a new header with schema version, category, name, and content type
    pub fn with_content_type(
        schema_version: String,
        schema_category: String,
        schema_name: String,
        content_type: String,
    ) -> Self {
        Self {
            schema_version,
            schema_category,
            schema_name,
            timestamp: Utc::now(),
            content_type: Some(content_type),
        }
    }

    /// Gets the schema version
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Gets the schema category
    pub fn schema_category(&self) -> &str {
        &self.schema_category
    }

    /// Gets the schema name
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    /// Gets the timestamp
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Gets the content type
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}
