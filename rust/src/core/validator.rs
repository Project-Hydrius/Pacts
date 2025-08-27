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

use crate::{Envelope, SchemaLoader};
use serde_json::Value;

/// Validation result containing validation status and errors
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Creates a new validation result
    pub fn new(valid: bool, errors: Vec<String>) -> Self {
        Self { valid, errors }
    }

    /// Creates a successful validation result
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    /// Creates a failed validation result with errors
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }

    /// Checks if validation was successful
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Gets the list of errors
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }

    /// Checks if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Gets the error message as a single string
    pub fn error_message(&self) -> String {
        if self.errors.is_empty() {
            "Validation successful".to_string()
        } else {
            self.errors.join("; ")
        }
    }
}

/// Validator class that validates data against schemas
#[derive(Clone)]
pub struct Validator {
    schema_loader: std::cell::RefCell<SchemaLoader>,
}

impl Validator {
    /// Creates a new validator with a schema loader
    pub fn new(schema_loader: SchemaLoader) -> Self {
        Self {
            schema_loader: std::cell::RefCell::new(schema_loader),
        }
    }

    /// Validates an envelope against its schema
    ///
    /// # Arguments
    /// * `envelope` - the envelope to validate
    ///
    /// # Returns
    /// the validation result
    pub fn validate(&mut self, envelope: &Envelope) -> ValidationResult {
        let mut errors = Vec::new();

        // Check if header is null (Rust doesn't have null, so we check if it's empty)
        if envelope.header.schema_category.is_empty()
            && envelope.header.schema_name.is_empty()
            && envelope.header.schema_version.is_empty()
        {
            errors.push("Header is required".to_string());
            return ValidationResult::new(false, errors);
        }

        // Validate schema category
        if envelope.header.schema_category.is_empty() {
            errors.push("Schema category is required in header".to_string());
        }

        // Validate schema name
        if envelope.header.schema_name.is_empty() {
            errors.push("Schema name is required in header".to_string());
        }

        // Validate schema version
        if envelope.header.schema_version.is_empty() {
            errors.push("Schema version is required in header".to_string());
        }

        // Load and validate schema if schema category and name are provided
        if !envelope.header.schema_category.is_empty() && !envelope.header.schema_name.is_empty() {
            let schema = self.schema_loader.borrow_mut().load_schema(
                &envelope.header.schema_category,
                &envelope.header.schema_name,
            );
            let data_validation = self.validate_data(&envelope.data, &schema);
            errors.extend(data_validation.get_errors().to_vec());
        }

        ValidationResult::new(errors.is_empty(), errors)
    }

    /// Validates data against a schema
    ///
    /// # Arguments
    /// * `data` - the data to validate (can be any serializable type)
    /// * `schema` - the JSON schema to validate against
    ///
    /// # Returns
    /// the validation result
    pub fn validate_data(&self, data: &Value, schema: &Value) -> ValidationResult {
        let mut errors = Vec::new();

        self.validate_required_fields(data, schema, &mut errors);
        self.validate_type_schema(data, schema, &mut errors);
        self.validate_properties(data, schema, &mut errors);

        ValidationResult::new(errors.is_empty(), errors)
    }

    /// Validates the type of a value
    fn validate_type(&self, data: &Value, expected_type: &str) -> bool {
        match expected_type {
            "object" => data.is_object(),
            "array" => data.is_array(),
            "string" => data.is_string(),
            "number" => data.is_number(),
            "boolean" => data.is_boolean(),
            "null" => data.is_null(),
            _ => true,
        }
    }

    /// Validates required fields
    fn validate_required_fields(&self, data: &Value, schema: &Value, errors: &mut Vec<String>) {
        if let Some(required_fields) = schema.get("required") {
            if let Some(required_array) = required_fields.as_array() {
                for field in required_array {
                    if let Some(field_name) = field.as_str() {
                        if !data.get(field_name).is_some() {
                            errors.push(format!("Required field missing: {}", field_name));
                        }
                    }
                }
            }
        }
    }

    /// Validates the type of the data against the schema
    fn validate_type_schema(&self, data: &Value, schema: &Value, errors: &mut Vec<String>) {
        if let Some(type_value) = schema.get("type") {
            if let Some(expected_type) = type_value.as_str() {
                if !self.validate_type(data, expected_type) {
                    errors.push(format!("Invalid type. Expected: {}", expected_type));
                }
            }
        }
    }

    /// Validates the type of a specific property
    fn validate_property_type(
        &self,
        data: &Value,
        property_name: &str,
        property_schema: &Value,
        errors: &mut Vec<String>,
    ) {
        if let Some(property_type) = property_schema.get("type") {
            if let Some(expected_type) = property_type.as_str() {
                if let Some(property_value) = data.get(property_name) {
                    if !self.validate_type(property_value, expected_type) {
                        errors.push(format!(
                            "Invalid type for field '{}'. Expected: {}",
                            property_name, expected_type
                        ));
                    }
                }
            }
        }
    }

    /// Validates properties of an object
    fn validate_properties(&self, data: &Value, schema: &Value, errors: &mut Vec<String>) {
        if let Some(properties) = schema.get("properties") {
            if data.is_object() && properties.is_object() {
                if let Some(properties_obj) = properties.as_object() {
                    for (property_name, property_schema) in properties_obj {
                        if data.get(property_name).is_some() {
                            self.validate_property_type(
                                data,
                                property_name,
                                property_schema,
                                errors,
                            );
                        }
                    }
                }
            }
        }
    }
}
