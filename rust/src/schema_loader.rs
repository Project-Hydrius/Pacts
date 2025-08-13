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

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// SchemaLoader struct that loads schemas from various sources
pub struct SchemaLoader {
    schema_cache: HashMap<String, Value>,
    schema_base_path: String,
}

impl SchemaLoader {
    /// Creates a new schema loader with default settings
    pub fn new() -> Self {
        Self {
            schema_cache: HashMap::new(),
            schema_base_path: "schemas".to_string(),
        }
    }

    /// Creates a new schema loader with a custom base path
    pub fn with_base_path(schema_base_path: String) -> Self {
        Self {
            schema_cache: HashMap::new(),
            schema_base_path,
        }
    }

    /// Loads a schema by ID and version
    pub fn load_schema(&mut self, schema_id: &str, version: &str) -> Option<Value> {
        let cache_key = format!("{}_{}", schema_id, version);

        if let Some(schema) = self.schema_cache.get(&cache_key) {
            return Some(schema.clone());
        }

        match self.load_schema_from_file(schema_id, version) {
            Ok(schema) => {
                self.schema_cache.insert(cache_key, schema.clone());
                Some(schema)
            }
            Err(_) => None,
        }
    }

    /// Loads a schema by directory structure: domain/category/schemaName
    /// The version is automatically extracted from the v{number} directory
    pub fn load_schema_by_directory(
        &mut self,
        domain: &str,
        category: &str,
        schema_name: &str,
    ) -> Option<Value> {
        let cache_key = format!("{}_{}_{}", domain, category, schema_name);

        if let Some(schema) = self.schema_cache.get(&cache_key) {
            return Some(schema.clone());
        }

        match self.load_schema_from_directory(domain, category, schema_name) {
            Ok(schema) => {
                self.schema_cache.insert(cache_key, schema.clone());
                Some(schema)
            }
            Err(_) => None,
        }
    }

    /// Loads a schema from directory structure
    pub fn load_schema_from_directory(
        &self,
        domain: &str,
        category: &str,
        schema_name: &str,
    ) -> Result<Value> {
        let domain_path = Path::new(&self.schema_base_path).join(domain);

        if !domain_path.exists() || !domain_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Domain directory not found: {:?}",
                domain_path
            ));
        }

        // Find the version directory (v{number})
        let version_regex = Regex::new(r"^v(\d+)$").unwrap();
        let version = fs::read_dir(&domain_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if version_regex.is_match(&name) {
                    Some(name)
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| anyhow::anyhow!("No version directory found in domain: {}", domain))?;

        // Construct the full path: schemas/domain/v{number}/category/schemaName.json
        let schema_path = Path::new(&self.schema_base_path)
            .join(domain)
            .join(&version)
            .join(category)
            .join(format!("{}.json", schema_name));

        if !schema_path.exists() {
            return Err(anyhow::anyhow!("Schema file not found: {:?}", schema_path));
        }

        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = serde_json::from_str(&schema_content)?;
        Ok(schema)
    }

    /// Loads a schema from a file
    pub fn load_schema_from_file(&self, schema_id: &str, version: &str) -> Result<Value> {
        let file_name = format!("{}_{}.json", schema_id, version);
        let schema_path = Path::new(&self.schema_base_path).join(file_name);

        if !schema_path.exists() {
            return Err(anyhow::anyhow!("Schema file not found: {:?}", schema_path));
        }

        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = serde_json::from_str(&schema_content)?;
        Ok(schema)
    }

    /// Loads a schema from a string
    pub fn load_schema_from_string(&self, schema_content: &str) -> Result<Value> {
        let schema: Value = serde_json::from_str(schema_content)?;
        Ok(schema)
    }

    /// Clears the schema cache
    pub fn clear_cache(&mut self) {
        self.schema_cache.clear();
    }

    /// Sets the base path for schema files
    pub fn set_schema_base_path(&mut self, schema_base_path: String) {
        self.schema_base_path = schema_base_path;
    }

    /// Gets the base path for schema files
    pub fn schema_base_path(&self) -> &str {
        &self.schema_base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_schema_structure(temp_dir: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Create domain directory
        let domain_path = temp_dir.path().join("test-domain");
        fs::create_dir_all(&domain_path)?;

        // Create version directory
        let version_path = domain_path.join("v1");
        fs::create_dir_all(&version_path)?;

        // Create category directory
        let category_path = version_path.join("test-category");
        fs::create_dir_all(&category_path)?;

        // Create test schema file
        let schema_content = r#"{
            "type": "object",
            "required": ["id", "name"],
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"},
                "description": {"type": "string"}
            }
        }"#;
        fs::write(category_path.join("test_schema.json"), schema_content)?;

        // Create another schema file
        let schema2_content = r#"{
            "type": "object",
            "required": ["code"],
            "properties": {
                "code": {"type": "number"},
                "status": {"type": "string"}
            }
        }"#;
        fs::write(category_path.join("another_schema.json"), schema2_content)?;

        Ok(())
    }

    #[test]
    fn test_new() {
        let schema_loader = SchemaLoader::new();
        assert_eq!(schema_loader.schema_base_path(), "schemas");
    }

    #[test]
    fn test_with_base_path() {
        let custom_path = "/custom/schemas/path".to_string();
        let schema_loader = SchemaLoader::with_base_path(custom_path.clone());
        assert_eq!(schema_loader.schema_base_path(), custom_path);
    }

    #[test]
    fn test_load_schema_by_directory_with_valid_schema() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema_by_directory("test-domain", "test-category", "test_schema");

        assert!(schema.is_some());
        if let Some(schema_value) = schema {
            assert!(schema_value.is_object());
            assert_eq!(schema_value["type"], "object");
            assert!(schema_value.get("required").is_some());
            assert!(schema_value.get("properties").is_some());
        }
    }

    #[test]
    fn test_load_schema_by_directory_with_non_existent_schema() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema_by_directory("test-domain", "test-category", "non_existent");

        assert!(schema.is_none());
    }

    #[test]
    fn test_load_schema_by_directory_with_non_existent_domain() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema_by_directory("non_existent", "test-category", "test_schema");

        assert!(schema.is_none());
    }

    #[test]
    fn test_load_schema_by_directory_with_non_existent_category() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema_by_directory("test-domain", "non_existent", "test_schema");

        assert!(schema.is_none());
    }

    #[test]
    fn test_load_schema_by_directory_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());

        // Load schema twice
        let schema1 = schema_loader.load_schema_by_directory("test-domain", "test-category", "test_schema");
        let schema2 = schema_loader.load_schema_by_directory("test-domain", "test-category", "test_schema");

        assert!(schema1.is_some());
        assert!(schema2.is_some());
        assert_eq!(schema1.unwrap(), schema2.unwrap());
    }

    #[test]
    fn test_load_schema_by_directory_with_multiple_schemas() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());

        let schema1 = schema_loader.load_schema_by_directory("test-domain", "test-category", "test_schema");
        let schema2 = schema_loader.load_schema_by_directory("test-domain", "test-category", "another_schema");

        assert!(schema1.is_some());
        assert!(schema2.is_some());
        assert_ne!(schema1.unwrap(), schema2.unwrap());
    }

    #[test]
    fn test_load_schema_from_directory_with_valid_path() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let result = schema_loader.load_schema_from_directory("test-domain", "test-category", "test_schema");

        assert!(result.is_ok());
        if let Ok(schema) = result {
            assert!(schema.is_object());
            assert_eq!(schema["type"], "object");
        }
    }

    #[test]
    fn test_load_schema_from_directory_with_non_existent_path() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let result = schema_loader.load_schema_from_directory("non_existent", "test-category", "test_schema");

        assert!(result.is_err());
    }

    #[test]
    fn test_load_schema_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let schema_content = r#"{
            "type": "object",
            "properties": {
                "test": {"type": "string"}
            }
        }"#;
        let schema_file = temp_dir.path().join("test_schema_1.0.json");
        fs::write(&schema_file, schema_content).unwrap();

        let schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let result = schema_loader.load_schema_from_file("test_schema", "1.0");

        assert!(result.is_ok());
        if let Ok(schema) = result {
            assert!(schema.is_object());
            assert_eq!(schema["type"], "object");
        }
    }

    #[test]
    fn test_load_schema_from_file_with_non_existent_file() {
        let temp_dir = TempDir::new().unwrap();
        let schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let result = schema_loader.load_schema_from_file("non_existent", "1.0");

        assert!(result.is_err());
    }

    #[test]
    fn test_load_schema_from_string() {
        let schema_loader = SchemaLoader::new();
        let schema_content = r#"{
            "type": "object",
            "properties": {
                "string_test": {"type": "boolean"}
            }
        }"#;

        let result = schema_loader.load_schema_from_string(schema_content);

        assert!(result.is_ok());
        if let Ok(schema) = result {
            assert!(schema.is_object());
            assert_eq!(schema["type"], "object");
        }
    }

    #[test]
    fn test_load_schema_from_string_with_invalid_json() {
        let schema_loader = SchemaLoader::new();
        let result = schema_loader.load_schema_from_string("invalid json content");

        assert!(result.is_err());
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());

        // Load a schema to populate cache
        let schema1 = schema_loader.load_schema_by_directory("test-domain", "test-category", "test_schema");
        assert!(schema1.is_some());

        // Clear cache
        schema_loader.clear_cache();

        // Load again - should still work but from file
        let schema2 = schema_loader.load_schema_by_directory("test-domain", "test-category", "test_schema");
        assert!(schema2.is_some());
        assert_eq!(schema1.unwrap(), schema2.unwrap());
    }

    #[test]
    fn test_set_and_get_schema_base_path() {
        let mut schema_loader = SchemaLoader::new();
        let new_path = "/new/schema/path".to_string();

        schema_loader.set_schema_base_path(new_path.clone());
        assert_eq!(schema_loader.schema_base_path(), new_path);
    }

    #[test]
    fn test_load_schema_with_version_and_id() {
        let temp_dir = TempDir::new().unwrap();
        let schema_content = r#"{
            "type": "object",
            "properties": {
                "version_test": {"type": "string"}
            }
        }"#;
        let schema_file = temp_dir.path().join("test_id_1.0.json");
        fs::write(&schema_file, schema_content).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema("test_id", "1.0");

        assert!(schema.is_some());
        if let Some(schema_value) = schema {
            assert!(schema_value.is_object());
            assert_eq!(schema_value["type"], "object");
        }
    }

    #[test]
    fn test_load_schema_with_non_existent_version_and_id() {
        let temp_dir = TempDir::new().unwrap();
        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema("non_existent", "1.0");

        assert!(schema.is_none());
    }

    #[test]
    fn test_load_schema_caching() {
        let temp_dir = TempDir::new().unwrap();
        let schema_content = r#"{
            "type": "object",
            "properties": {
                "cache_test": {"type": "string"}
            }
        }"#;
        let schema_file = temp_dir.path().join("cache_test_1.0.json");
        fs::write(&schema_file, schema_content).unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());

        // Load schema twice
        let schema1 = schema_loader.load_schema("cache_test", "1.0");
        let schema2 = schema_loader.load_schema("cache_test", "1.0");

        assert!(schema1.is_some());
        assert!(schema2.is_some());
        assert_eq!(schema1.unwrap(), schema2.unwrap());
    }

    #[test]
    fn test_with_invalid_schema_content() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        // Create a schema file with invalid JSON
        let schema_file = temp_dir.path().join("test-domain").join("v1").join("test-category").join("invalid_schema.json");
        fs::write(&schema_file, "invalid json content").unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema_by_directory("test-domain", "test-category", "invalid_schema");

        // Should return None for invalid JSON
        assert!(schema.is_none());
    }

    #[test]
    fn test_with_empty_schema_file() {
        let temp_dir = TempDir::new().unwrap();
        create_test_schema_structure(&temp_dir).unwrap();

        // Create an empty schema file
        let schema_file = temp_dir.path().join("test-domain").join("v1").join("test-category").join("empty_schema.json");
        fs::write(&schema_file, "").unwrap();

        let mut schema_loader = SchemaLoader::with_base_path(temp_dir.path().to_string_lossy().to_string());
        let schema = schema_loader.load_schema_by_directory("test-domain", "test-category", "empty_schema");

        // Should return None for empty file
        assert!(schema.is_none());
    }
}
