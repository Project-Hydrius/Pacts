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
    /// Creates a new PactsService
    pub fn new(schema_root: String, domain: String, version: String) -> Self {
        let schema_loader = SchemaLoader::new(schema_root, domain, version);
        let validator = Validator::new(schema_loader.clone());

        Self {
            validator: Arc::new(validator),
            schema_loader: Arc::new(RefCell::new(schema_loader)),
        }
    }

    /// Creates an envelope
    pub fn create_envelope(
        &self,
        schema_category: String,
        schema_name: String,
        data: Value,
    ) -> Envelope {
        let header = Header::with_content_type(
            self.schema_loader.borrow().get_version().to_string(),
            schema_category,
            schema_name,
            "application/json".to_string(),
        );
        Envelope::new(header, data)
    }

    /// Validates an envelope
    pub fn validate(&self, envelope: &Envelope) -> ValidationResult {
        // We need to clone the validator to get a mutable reference
        let mut validator = (*self.validator).clone();
        validator.validate(envelope)
    }

    /// Validates data against a specific schema
    pub fn validate_data(
        &self,
        data: &Value,
        category: &str,
        schema_name: &str,
    ) -> ValidationResult {
        match self
            .schema_loader
            .borrow_mut()
            .load_schema(category, schema_name)
        {
            schema => {
                let validator = (*self.validator).clone();
                validator.validate_data(data, &schema)
            }
        }
    }

    /// Sends validated data using a provided sender function
    pub fn send_validated_data<T, F>(
        &self,
        schema_category: String,
        schema_name: String,
        data: Value,
        sender: F,
    ) -> Result<T, String>
    where
        F: FnOnce(&Envelope) -> Result<T, String>,
    {
        let envelope =
            self.create_envelope(schema_category, schema_name, data);
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
        Self::new("schemas".to_string(), "bees".to_string(), "v1".to_string())
    }
}
