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

pub mod core;
pub mod r#impl;
pub mod model;

pub use crate::r#impl::PactsService;
pub use core::schema_loader::SchemaLoader;
pub use core::validator::{ValidationResult, Validator};
pub use model::Envelope;
pub use model::Header;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_library_exports() {
        // Test that all public items are accessible
        let _envelope: Envelope = Envelope::new(
            Header::new("1.0".to_string(), "test".to_string(), "test".to_string()),
            json!({}),
        );

        let _header: Header =
            Header::new("1.0".to_string(), "test".to_string(), "test".to_string());
        let _schema_loader: SchemaLoader =
            SchemaLoader::new("schemas".to_string(), "bees".to_string(), "v1".to_string());
        let _validator: Validator = Validator::new(_schema_loader.clone());
        let _validation_result: ValidationResult = ValidationResult::success();

        // If we get here, all exports are working
        assert!(true);
    }

    #[test]
    fn test_basic_workflow() {
        // Test a complete workflow from envelope creation to validation
        let header = Header::new(
            "1.0".to_string(),
            "test".to_string(),
            "test-schema".to_string(),
        );
        let data = json!({
            "id": "123",
            "name": "Test Item"
        });

        let envelope = Envelope::new(header, data);
        let schema_loader =
            SchemaLoader::new("schemas".to_string(), "bees".to_string(), "v1".to_string());
        let mut validator = Validator::new(schema_loader);

        let result = validator.validate(&envelope);

        // Should fail because schema doesn't exist, but the workflow should complete
        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert!(result.error_message().contains("Schema not found"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let header = Header::new(
            "1.0".to_string(),
            "test".to_string(),
            "test-schema".to_string(),
        );
        let data = json!({
            "id": "123",
            "name": "Test Item"
        });
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("source".to_string(), json!("test"));

        let envelope = Envelope::with_metadata(header, data, metadata);

        // Serialize and deserialize
        let serialized = serde_json::to_string(&envelope).unwrap();
        let deserialized: Envelope = serde_json::from_str(&serialized).unwrap();

        // Verify the roundtrip worked
        assert_eq!(
            envelope.header.schema_version,
            deserialized.header.schema_version
        );
        assert_eq!(
            envelope.header.schema_category,
            deserialized.header.schema_category
        );
        assert_eq!(envelope.header.schema_name, deserialized.header.schema_name);
        assert_eq!(envelope.data, deserialized.data);
        assert_eq!(envelope.metadata, deserialized.metadata);
    }

    #[test]
    fn test_validation_result_methods() {
        let success_result = ValidationResult::success();
        assert!(success_result.is_valid());
        assert!(!success_result.has_errors());
        assert_eq!(success_result.error_message(), "Validation successful");

        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let failure_result = ValidationResult::failure(errors.clone());
        assert!(!failure_result.is_valid());
        assert!(failure_result.has_errors());
        assert_eq!(failure_result.get_errors(), &errors);
        assert_eq!(failure_result.error_message(), "Error 1; Error 2");
    }

    #[test]
    fn test_schema_loader_basic_operations() {
        let mut schema_loader =
            SchemaLoader::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        // Test basic operations
        assert_eq!(schema_loader.get_schema_root(), "schemas");

        schema_loader.clear_cache();
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_header_timestamp_consistency() {
        let header1 = Header::new("1.0".to_string(), "test".to_string(), "test".to_string());
        let header2 = Header::new("1.0".to_string(), "test".to_string(), "test".to_string());

        // Both should have recent timestamps
        let now = chrono::Utc::now();
        assert!(header1.timestamp() > &(now - chrono::Duration::seconds(5)));
        assert!(header2.timestamp() > &(now - chrono::Duration::seconds(5)));

        // Timestamps should be different
        assert_ne!(header1.timestamp(), header2.timestamp());
    }

    #[test]
    fn test_envelope_with_different_data_types() {
        let header = Header::new("1.0".to_string(), "test".to_string(), "test".to_string());

        // Test with different data types
        let _envelope1 = Envelope::new(header.clone(), json!("string data"));
        let _envelope2 = Envelope::new(header.clone(), json!(42));
        let _envelope3 = Envelope::new(header.clone(), json!(true));
        let _envelope4 = Envelope::new(header.clone(), json!(null));
        let _envelope5 = Envelope::new(header.clone(), json!([1, 2, 3]));
        let _envelope6 = Envelope::new(header, json!({"key": "value"}));

        // If we get here, all data types work
        assert!(true);
    }

    #[test]
    fn test_validation_result_edge_cases() {
        // Test with empty errors
        let result = ValidationResult::new(false, Vec::new());
        assert!(!result.is_valid());
        assert!(!result.has_errors());
        assert_eq!(result.error_message(), "Validation successful");

        // Test with single error
        let result = ValidationResult::failure(vec!["Single error".to_string()]);
        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert_eq!(result.error_message(), "Single error");
    }

    #[test]
    fn test_clone_operations() {
        let header = Header::new("1.0".to_string(), "test".to_string(), "test".to_string());
        let data = json!({"id": "123"});
        let envelope = Envelope::new(header, data);

        let cloned_envelope = envelope.clone();
        assert_eq!(
            envelope.header.schema_version,
            cloned_envelope.header.schema_version
        );
        assert_eq!(
            envelope.header.schema_category,
            cloned_envelope.header.schema_category
        );
        assert_eq!(
            envelope.header.schema_name,
            cloned_envelope.header.schema_name
        );
        assert_eq!(envelope.data, cloned_envelope.data);

        let validation_result = ValidationResult::success();
        let cloned_result = validation_result.clone();
        assert_eq!(validation_result.valid, cloned_result.valid);
        assert_eq!(validation_result.errors, cloned_result.errors);
    }
}
