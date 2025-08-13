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
use std::cell::RefCell;

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
    pub fn errors(&self) -> &[String] {
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

/// Validator struct that validates data against schemas
pub struct Validator {
    schema_loader: RefCell<SchemaLoader>,
}

impl Validator {
    /// Creates a new validator with default schema loader
    pub fn new() -> Self {
        Self {
            schema_loader: RefCell::new(SchemaLoader::new()),
        }
    }

    /// Creates a new validator with a custom schema loader
    pub fn with_schema_loader(schema_loader: SchemaLoader) -> Self {
        Self {
            schema_loader: RefCell::new(schema_loader),
        }
    }

    /// Validates an envelope against its schema
    pub fn validate(&self, envelope: &Envelope) -> ValidationResult {
        let mut errors = Vec::new();

        // Validate header
        if envelope.header.schema_id.is_empty() {
            errors.push("Schema ID is required in header".to_string());
        }

        if envelope.header.schema_version.is_empty() {
            errors.push("Schema version is required in header".to_string());
        }

        // Load and validate schema
        if !envelope.header.schema_id.is_empty() {
            match self
                .schema_loader
                .borrow_mut()
                .load_schema(&envelope.header.schema_id, &envelope.header.schema_version)
            {
                Some(schema) => {
                    let data_validation = self.validate_data(&envelope.data, &schema);
                    errors.extend(data_validation.errors);
                }
                None => {
                    errors.push(format!(
                        "Schema not found: {} version {}",
                        envelope.header.schema_id, envelope.header.schema_version
                    ));
                }
            }
        }

        ValidationResult::new(errors.is_empty(), errors)
    }

    /// Validates data against a schema
    pub fn validate_data(&self, data: &Value, schema: &Value) -> ValidationResult {
        let mut errors = Vec::new();

        // Basic required field validation
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

        // Basic type validation
        if let Some(type_value) = schema.get("type") {
            if let Some(expected_type) = type_value.as_str() {
                if !self.validate_type(data, expected_type) {
                    errors.push(format!("Invalid type. Expected: {}", expected_type));
                }
            }
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Envelope, Header};
    use serde_json::json;

    #[test]
    fn test_validation_result_new() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::new(false, errors.clone());

        assert!(!result.is_valid());
        assert_eq!(result.errors(), &errors);
        assert!(result.has_errors());
    }

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();

        assert!(result.is_valid());
        assert!(result.errors().is_empty());
        assert!(!result.has_errors());
        assert_eq!(result.error_message(), "Validation successful");
    }

    #[test]
    fn test_validation_result_failure() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::failure(errors.clone());

        assert!(!result.is_valid());
        assert_eq!(result.errors(), &errors);
        assert!(result.has_errors());
        assert_eq!(result.error_message(), "Error 1; Error 2");
    }

    #[test]
    fn test_validation_result_single_error() {
        let errors = vec!["Single error".to_string()];
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert_eq!(result.error_message(), "Single error");
    }

    #[test]
    fn test_validation_result_empty_errors() {
        let result = ValidationResult::new(false, Vec::new());

        assert!(!result.is_valid());
        assert!(!result.has_errors());
        assert_eq!(result.error_message(), "Validation successful");
    }

    #[test]
    fn test_validation_result_clone() {
        let errors = vec!["Error 1".to_string()];
        let result = ValidationResult::failure(errors);
        let cloned = result.clone();

        assert_eq!(result.valid, cloned.valid);
        assert_eq!(result.errors, cloned.errors);
    }

    #[test]
    fn test_validator_new() {
        let validator = Validator::new();
        assert!(validator.schema_loader.borrow().schema_base_path() == "schemas");
    }

    #[test]
    fn test_validator_with_schema_loader() {
        let schema_loader = SchemaLoader::with_base_path("custom/path".to_string());
        let validator = Validator::with_schema_loader(schema_loader);
        assert!(validator.schema_loader.borrow().schema_base_path() == "custom/path");
    }

    #[test]
    fn test_validate_envelope_with_empty_schema_id() {
        let validator = Validator::new();
        let header = Header::new("1.0".to_string(), "".to_string());
        let envelope = Envelope::new(header, json!({}));

        let result = validator.validate(&envelope);

        assert!(!result.is_valid());
        assert!(result
            .errors()
            .contains(&"Schema ID is required in header".to_string()));
    }

    #[test]
    fn test_validate_envelope_with_empty_schema_version() {
        let validator = Validator::new();
        let header = Header::new("".to_string(), "test-schema".to_string());
        let envelope = Envelope::new(header, json!({}));

        let result = validator.validate(&envelope);

        assert!(!result.is_valid());
        assert!(result
            .errors()
            .contains(&"Schema version is required in header".to_string()));
    }

    #[test]
    fn test_validate_envelope_with_non_existent_schema() {
        let validator = Validator::new();
        let header = Header::new("1.0".to_string(), "non-existent-schema".to_string());
        let envelope = Envelope::new(header, json!({}));

        let result = validator.validate(&envelope);

        assert!(!result.is_valid());
        assert!(result
            .errors()
            .contains(&"Schema not found: non-existent-schema version 1.0".to_string()));
    }

    #[test]
    fn test_validate_data_with_valid_object() {
        let validator = Validator::new();
        let schema = json!({
            "type": "object",
            "required": ["id", "name"],
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"}
            }
        });
        let data = json!({
            "id": "123",
            "name": "Test Item"
        });

        let result = validator.validate_data(&data, &schema);

        assert!(result.is_valid());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validate_data_with_missing_required_field() {
        let validator = Validator::new();
        let schema = json!({
            "type": "object",
            "required": ["id", "name"],
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"}
            }
        });
        let data = json!({
            "id": "123"
            // Missing "name" field
        });

        let result = validator.validate_data(&data, &schema);

        assert!(!result.is_valid());
        assert!(result
            .errors()
            .contains(&"Required field missing: name".to_string()));
    }

    #[test]
    fn test_validate_data_with_wrong_type() {
        let validator = Validator::new();
        let schema = json!({
            "type": "array",  // Expecting array but getting object
            "required": ["id"]
        });
        let data = json!({
            "id": "123"  // This is an object, not an array
        });

        let result = validator.validate_data(&data, &schema);

        assert!(!result.is_valid());
        assert!(result
            .errors()
            .contains(&"Invalid type. Expected: array".to_string()));
    }

    #[test]
    fn test_validate_type_object() {
        let validator = Validator::new();
        let data = json!({"key": "value"});

        assert!(validator
            .validate_data(&data, &json!({"type": "object"}))
            .is_valid());
    }

    #[test]
    fn test_validate_type_array() {
        let validator = Validator::new();
        let data = json!([1, 2, 3]);

        assert!(validator
            .validate_data(&data, &json!({"type": "array"}))
            .is_valid());
    }

    #[test]
    fn test_validate_type_string() {
        let validator = Validator::new();
        let data = json!("test string");

        assert!(validator
            .validate_data(&data, &json!({"type": "string"}))
            .is_valid());
    }

    #[test]
    fn test_validate_type_number() {
        let validator = Validator::new();
        let data = json!(42);

        assert!(validator
            .validate_data(&data, &json!({"type": "number"}))
            .is_valid());
    }

    #[test]
    fn test_validate_type_boolean() {
        let validator = Validator::new();
        let data = json!(true);

        assert!(validator
            .validate_data(&data, &json!({"type": "boolean"}))
            .is_valid());
    }

    #[test]
    fn test_validate_type_null() {
        let validator = Validator::new();
        let data = json!(null);

        assert!(validator
            .validate_data(&data, &json!({"type": "null"}))
            .is_valid());
    }

    #[test]
    fn test_validate_data_with_no_required_fields() {
        let validator = Validator::new();
        let schema = json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"}
            }
        });
        let data = json!({});

        let result = validator.validate_data(&data, &schema);

        assert!(result.is_valid());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validate_data_with_no_type_specified() {
        let validator = Validator::new();
        let schema = json!({
            "required": ["id"]
        });
        let data = json!({
            "id": "123"
        });

        let result = validator.validate_data(&data, &schema);

        assert!(result.is_valid());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validate_data_with_complex_nested_object() {
        let validator = Validator::new();
        let schema = json!({
            "type": "object",
            "required": ["user"],
            "properties": {
                "user": {"type": "object"}
            }
        });
        let data = json!({
            "user": {
                "name": "John",
                "age": 30
            }
        });

        let result = validator.validate_data(&data, &schema);

        assert!(result.is_valid());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validate_data_with_array() {
        let validator = Validator::new();
        let schema = json!({
            "type": "object",
            "required": ["items"],
            "properties": {
                "items": {"type": "array"}
            }
        });
        let data = json!({
            "items": ["item1", "item2", "item3"]
        });

        let result = validator.validate_data(&data, &schema);

        assert!(result.is_valid());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validation_result_with_special_characters() {
        let errors = vec![
            "Error with special chars: !@#$%^&*()".to_string(),
            "Error with unicode: 测试错误".to_string(),
            "Error with quotes: \"quoted error\"".to_string(),
        ];
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert_eq!(result.errors().len(), 3);
        assert!(result.error_message().contains("Error with special chars"));
        assert!(result.error_message().contains("测试错误"));
        assert!(result.error_message().contains("quoted error"));
    }
}
