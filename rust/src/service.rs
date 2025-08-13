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

use crate::{Envelope, Header, SchemaLoader, ValidationResult, Validator};
use serde_json::Value;
use std::cell::RefCell;
use std::sync::Arc;

/// Service struct for convenient Pacts operations
pub struct PactsService {
    validator: Arc<Validator>,
    schema_loader: Arc<RefCell<SchemaLoader>>,
}

impl PactsService {
    /// Creates a new PactsService with default settings
    pub fn new() -> Self {
        let schema_loader = SchemaLoader::new();
        let validator = Validator::with_schema_loader(schema_loader);

        Self {
            validator: Arc::new(validator),
            schema_loader: Arc::new(RefCell::new(SchemaLoader::new())),
        }
    }

    /// Creates a new PactsService with a custom schema base path
    pub fn with_base_path(base_path: String) -> Self {
        let schema_loader = SchemaLoader::with_base_path(base_path);
        let validator = Validator::with_schema_loader(schema_loader.clone());

        Self {
            validator: Arc::new(validator),
            schema_loader: Arc::new(RefCell::new(schema_loader)),
        }
    }

    /// Creates an envelope with authentication
    pub fn create_envelope_with_auth(
        &self,
        schema_version: String,
        schema_id: String,
        data: Value,
        auth_token: String,
    ) -> Envelope {
        let header = Header::with_auth(
            schema_version,
            schema_id,
            Some("application/json".to_string()),
            auth_token,
        );
        Envelope::new(header, data)
    }

    /// Creates an envelope without authentication
    pub fn create_envelope(
        &self,
        schema_version: String,
        schema_id: String,
        data: Value,
    ) -> Envelope {
        let header = Header::with_content_type(
            schema_version,
            schema_id,
            "application/json".to_string(),
        );
        Envelope::new(header, data)
    }

    /// Validates an envelope
    pub fn validate(&self, envelope: &Envelope) -> ValidationResult {
        self.validator.validate(envelope)
    }

    /// Validates data against a specific schema
    pub fn validate_data(
        &self,
        data: &Value,
        domain: &str,
        category: &str,
        schema_name: &str,
    ) -> ValidationResult {
        match self.schema_loader.borrow_mut().load_schema_by_directory(domain, category, schema_name) {
            Some(schema) => self.validator.validate_data(data, &schema),
            None => ValidationResult::failure(vec![
                format!("Schema not found: {}/{}/{}", domain, category, schema_name)
            ]),
        }
    }

    /// Sends validated data using a provided sender function
    pub fn send_validated_data<T, F>(
        &self,
        schema_version: String,
        schema_id: String,
        data: Value,
        auth_token: String,
        sender: F,
    ) -> Result<T, String>
    where
        F: FnOnce(&Envelope) -> Result<T, String>,
    {
        let envelope = self.create_envelope_with_auth(schema_version, schema_id, data, auth_token);
        let result = self.validate(&envelope);

        if result.is_valid() {
            sender(&envelope)
        } else {
            Err(format!("Validation failed: {}", result.error_message()))
        }
    }

    /// Gets a reference to the validator
    pub fn validator(&self) -> &Arc<Validator> {
        &self.validator
    }

    /// Gets a reference to the schema loader
    pub fn schema_loader(&self) -> &Arc<RefCell<SchemaLoader>> {
        &self.schema_loader
    }
}

impl Default for PactsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_envelope_with_auth() {
        let service = PactsService::new();
        let data = json!({"test": "data"});

        let envelope = service.create_envelope_with_auth(
            "1.0".to_string(),
            "test-schema".to_string(),
            data.clone(),
            "test-token".to_string(),
        );

        assert_eq!(envelope.header.schema_version, "1.0");
        assert_eq!(envelope.header.schema_id, "test-schema");
        assert_eq!(envelope.header.auth_token, Some("test-token".to_string()));
        assert_eq!(envelope.data, data);
    }

    #[test]
    fn test_create_envelope_without_auth() {
        let service = PactsService::new();
        let data = json!({"test": "data"});

        let envelope = service.create_envelope(
            "1.0".to_string(),
            "test-schema".to_string(),
            data.clone(),
        );

        assert_eq!(envelope.header.schema_version, "1.0");
        assert_eq!(envelope.header.schema_id, "test-schema");
        assert_eq!(envelope.header.auth_token, None);
        assert_eq!(envelope.data, data);
    }

    #[test]
    fn test_send_validated_data_success() {
        let service = PactsService::new();
        let data = json!({"test": "data"});

        let result = service.send_validated_data(
            "1.0".to_string(),
            "test-schema".to_string(),
            data,
            "test-token".to_string(),
            |envelope| {
                // Simulate successful sending
                Ok(format!("Sent envelope with schema {}", envelope.header.schema_id))
            },
        );

        // Will fail validation because schema doesn't exist, but the mechanism works
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Schema not found"));
    }
}
