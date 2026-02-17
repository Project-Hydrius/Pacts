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
            Header::new("v1".to_string(), "test".to_string(), "test".to_string()),
            json!({}),
        );

        let _header: Header = Header::new("v1".to_string(), "test".to_string(), "test".to_string());
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
}
