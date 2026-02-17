pub mod core;
pub mod r#impl;
pub mod model;

pub use crate::r#impl::PactsService;
pub use core::schema_loader::SchemaLoader;
pub use core::validator::{ValidationResult, Validator};
pub use model::Envelope;
pub use model::Header;

/// Initializes the logging system for the pacts library.
/// This should be called once at the start of your application.
/// It uses env_logger with default settings.
pub fn init_logging() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default()).try_init();
}

/// Initializes the logging system with custom default filter level.
/// Example: `init_logging_with_level("info")` or `init_logging_with_level("debug")`
pub fn init_logging_with_level(level: &str) {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level))
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_logging() {
        INIT.call_once(|| {
            let _ =
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                    .is_test(true)
                    .try_init();
        });
    }

    #[test]
    fn test_library_exports() {
        init_test_logging();

        let _envelope: Envelope = Envelope::new(
            Header::new("v1".to_string(), "test".to_string(), "test".to_string()),
            json!({}),
        );

        let _header: Header = Header::new("v1".to_string(), "test".to_string(), "test".to_string());
        let _schema_loader: SchemaLoader =
            SchemaLoader::new("schemas".to_string(), "bees".to_string(), "v1".to_string());
        let _validator: Validator = Validator::new(_schema_loader.clone());
        let _validation_result: ValidationResult = ValidationResult::success();

        assert!(true);
    }

    #[test]
    fn test_basic_workflow() {
        init_test_logging();

        let header = Header::new(
            "v1".to_string(),
            "inventory".to_string(),
            "inventory_item".to_string(),
        );
        let data = json!({
            "slot": 1,
            "material": "Paper",
            "amount": 2
        });

        let envelope = Envelope::new(header, data);
        let schema_loader =
            SchemaLoader::new("schemas".to_string(), "bees".to_string(), "v1".to_string());
        let mut validator = Validator::new(schema_loader);

        let result = validator.validate(&envelope);

        assert!(result.is_valid());
    }

    #[test]
    fn test_schema_loader_initialization() {
        init_test_logging();

        let schema_loader =
            SchemaLoader::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        assert_eq!("schemas", schema_loader.get_schema_root());
        assert_eq!("bees", schema_loader.get_domain());
        assert_eq!("v1", schema_loader.get_version());
        assert_eq!(1, schema_loader.get_parsed_version());
    }

    #[test]
    fn test_pacts_service_create_envelope() {
        init_test_logging();

        let service =
            PactsService::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        let player_data = json!({
            "target_id": "player-123",
            "request_type": "PLAYER_JOIN",
            "date": "2025-01-01"
        });

        let envelope = service.create_envelope(
            "player".to_string(),
            "player_request".to_string(),
            player_data.clone(),
        );

        assert_eq!("v1", envelope.header.schema_version());
        assert_eq!("player", envelope.header.schema_category());
        assert_eq!("player_request", envelope.header.schema_name());
        assert_eq!(player_data, *envelope.data());
    }

    #[test]
    fn test_validate_envelope_with_null_header() {
        init_test_logging();

        let service =
            PactsService::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        let envelope = Envelope::new(
            Header::new("".to_string(), "".to_string(), "".to_string()),
            json!({}),
        );

        let result = service.validate(&envelope);

        assert!(!result.is_valid());
        assert_eq!(1, result.get_errors().len());
        assert_eq!("Header is required", result.get_errors()[0]);
    }

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.is_valid());
        assert!(!result.has_errors());
        assert!(result.get_errors().is_empty());
        assert_eq!("Validation successful", result.error_message());
    }

    #[test]
    fn test_validation_result_failure() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::failure(errors.clone());

        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert_eq!(2, result.get_errors().len());
        assert_eq!("Error 1; Error 2", result.error_message());
    }

    #[test]
    fn test_validation_result_new() {
        let errors = vec!["Test error".to_string()];
        let result = ValidationResult::new(false, errors);

        assert!(!result.is_valid());
        assert_eq!(1, result.get_errors().len());
    }

    #[test]
    fn test_envelope_with_metadata() {
        use std::collections::HashMap;

        let header = Header::new("v1".to_string(), "test".to_string(), "test".to_string());
        let data = json!({"key": "value"});
        let mut metadata = HashMap::new();
        metadata.insert("meta_key".to_string(), json!("meta_value"));

        let envelope = Envelope::with_metadata(header, data, metadata);

        assert!(envelope.metadata().is_some());
        assert_eq!(
            json!("meta_value"),
            *envelope.metadata().unwrap().get("meta_key").unwrap()
        );
    }

    #[test]
    fn test_envelope_without_metadata() {
        let header = Header::new("v1".to_string(), "test".to_string(), "test".to_string());
        let data = json!({"key": "value"});

        let envelope = Envelope::new(header, data);

        assert!(envelope.metadata().is_none());
    }

    #[test]
    fn test_header_with_content_type() {
        let header = Header::with_content_type(
            "v1".to_string(),
            "category".to_string(),
            "name".to_string(),
            "application/json".to_string(),
        );

        assert_eq!("v1", header.schema_version());
        assert_eq!("category", header.schema_category());
        assert_eq!("name", header.schema_name());
    }

    #[test]
    fn test_header_timestamp() {
        let header = Header::new("v1".to_string(), "test".to_string(), "test".to_string());

        // Just verify we can access the timestamp (it's always set on new headers)
        let _timestamp = header.timestamp();
        assert!(true);
    }

    #[test]
    fn test_send_validated_data_success() {
        init_test_logging();

        let service =
            PactsService::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        let data = json!({
            "slot": 1,
            "material": "Paper",
            "amount": 2
        });

        let result = service.send_validated_data(
            "inventory".to_string(),
            "inventory_item".to_string(),
            data,
            |_envelope: &Envelope| -> Result<String, String> {
                Ok("Sent successfully".to_string())
            },
        );

        assert!(result.is_ok());
        assert_eq!("Sent successfully", result.unwrap());
    }

    #[test]
    fn test_send_validated_data_failure() {
        init_test_logging();

        let service =
            PactsService::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        let data = json!({"invalid": "data"});

        let result = service.send_validated_data(
            "".to_string(),
            "".to_string(),
            data,
            |_envelope: &Envelope| -> Result<String, String> {
                Ok("Should not reach here".to_string())
            },
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Validation failed"));
    }

    #[test]
    fn test_validate_data_directly() {
        init_test_logging();

        let service =
            PactsService::new("schemas".to_string(), "bees".to_string(), "v1".to_string());

        let data = json!({
            "slot": 1,
            "material": "Paper",
            "amount": 2
        });

        let result = service.validate_data(&data, "inventory", "inventory_item");

        assert!(result.is_valid());
    }

    #[test]
    fn test_pacts_service_default() {
        init_test_logging();

        let service = PactsService::default();

        assert_eq!(
            "schemas",
            service.schema_loader().borrow().get_schema_root()
        );
        assert_eq!("bees", service.schema_loader().borrow().get_domain());
        assert_eq!("v1", service.schema_loader().borrow().get_version());
    }

    #[test]
    fn test_header_getters() {
        let header = Header::new(
            "v1".to_string(),
            "player".to_string(),
            "player_request".to_string(),
        );

        assert_eq!("v1", header.schema_version());
        assert_eq!("player", header.schema_category());
        assert_eq!("player_request", header.schema_name());
    }
}
