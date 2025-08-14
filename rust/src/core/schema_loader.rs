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
use std::io::Read;
use std::path::Path;

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../schemas"]
struct EmbeddedSchemas;

/// SchemaLoader struct that loads schemas that are bundled with Pacts.
#[derive(Clone)]
pub struct SchemaLoader {
    schema_cache: HashMap<String, Value>,
    schema_root: String,
    domain: String,
    version: String,
}

impl SchemaLoader {
    /// Creates a new SchemaLoader.
    ///
    /// # Arguments
    /// * `schema_root` - the directory containing the schemas
    /// * `domain` - the domain of the schema
    /// * `version` - the version of the schema
    pub fn new(schema_root: String, domain: String, version: String) -> Self {
        if schema_root.is_empty() || domain.is_empty() || version.is_empty() {
            panic!("Schema root, domain, and version must be specified.");
        }

        Self {
            schema_cache: HashMap::new(),
            schema_root,
            domain,
            version,
        }
    }

    /// Loads a schema from cache, file system, or classpath by category and name.
    ///
    /// # Arguments
    /// * `category` - The schema category (e.g., "player")
    /// * `name` - The schema name (e.g., "player_request")
    ///
    /// # Returns
    /// The parsed JSON schema value
    ///
    /// # Panics
    /// Panics if the schema cannot be loaded
    pub fn load_schema(&mut self, category: &str, name: &str) -> Value {
        let cache_key = format!("{}/{}/{}/{}", self.domain, self.version, category, name);

        if let Some(schema) = self.schema_cache.get(&cache_key) {
            return schema.clone();
        }

        match self.load_schema_internal(category, name) {
            Ok(schema) => {
                self.schema_cache.insert(cache_key, schema.clone());
                schema
            }
            Err(e) => {
                panic!(
                    "Failed to load schema: {}/{}/{}/{} - {}",
                    self.domain, self.version, category, name, e
                );
            }
        }
    }

    /// Attempts to load schema from file system, then embedded resources.
    ///
    /// # Arguments
    /// * `category` - the category of the schema
    /// * `name` - the name of the schema
    ///
    /// # Returns
    /// Result containing the schema or an error
    fn load_schema_internal(&self, category: &str, name: &str) -> Result<Value> {
        let file_path = Path::new(&self.schema_root)
            .join(&self.domain)
            .join(&self.version)
            .join(category)
            .join(format!("{}.json", name));

        if file_path.exists() {
            let schema_content = fs::read_to_string(file_path)?;
            let schema: Value = serde_json::from_str(&schema_content)?;
            return Ok(schema);
        }

        // Fallback to embedded resource
        let resource_path = format!(
            "{}/{}/{}/{}.json",
            self.domain, self.version, category, name
        );
        if let Some(file) = EmbeddedSchemas::get(&resource_path) {
            let schema: Value = serde_json::from_slice(&file.data)?;
            return Ok(schema);
        }

        Err(anyhow::anyhow!("Schema not found: {}", resource_path))
    }

    /// Loads a schema from a raw string.
    ///
    /// # Arguments
    /// * `schema_content` - the JSON schema content as a string
    ///
    /// # Returns
    /// Result containing the parsed schema or an error
    pub fn load_schema_from_string(&self, schema_content: &str) -> Result<Value> {
        let schema: Value = serde_json::from_str(schema_content)?;
        Ok(schema)
    }

    /// Loads a schema from a raw input stream.
    ///
    /// # Arguments
    /// * `input_stream` - the input stream containing JSON schema content
    ///
    /// # Returns
    /// Result containing the parsed schema or an error
    pub fn load_schema_from_stream<R: Read>(&self, mut input_stream: R) -> Result<Value> {
        let mut content = String::new();
        input_stream.read_to_string(&mut content)?;
        let schema: Value = serde_json::from_str(&content)?;
        Ok(schema)
    }

    /// Clears all cached schemas.
    pub fn clear_cache(&mut self) {
        self.schema_cache.clear();
    }

    /// Gets the schema root.
    ///
    /// # Returns
    /// the schema root
    pub fn get_schema_root(&self) -> &str {
        &self.schema_root
    }

    /// Gets the domain.
    ///
    /// # Returns
    /// the domain
    pub fn get_domain(&self) -> &str {
        &self.domain
    }

    /// Gets the version.
    ///
    /// # Returns
    /// the version
    pub fn get_version(&self) -> &str {
        &self.version
    }

    /// Gets the parsed version.
    ///
    /// # Returns
    /// the parsed version as an integer
    pub fn get_parsed_version(&self) -> i32 {
        self.version.replace("v", "").parse().unwrap_or(1)
    }

    // Legacy methods for backward compatibility
    #[deprecated(since = "1.0.0", note = "Use new() constructor instead")]
    pub fn with_base_path(schema_base_path: String) -> Self {
        Self {
            schema_cache: HashMap::new(),
            schema_root: schema_base_path,
            domain: String::new(),
            version: String::new(),
        }
    }

    #[deprecated(since = "1.0.0", note = "Use new() constructor instead")]
    pub fn with_version(schema_base_path: String, version_directory: String) -> Self {
        Self {
            schema_cache: HashMap::new(),
            schema_root: schema_base_path,
            domain: String::new(),
            version: version_directory,
        }
    }

    /// Loads a schema by ID and version (legacy method)
    #[deprecated(since = "1.0.0", note = "Use load_schema() instead")]
    pub fn load_schema_by_id(&mut self, schema_id: &str, version: &str) -> Option<Value> {
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

    /// Loads a schema by directory structure: domain/category/schemaName (legacy method)
    #[deprecated(since = "1.0.0", note = "Use load_schema() instead")]
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

    /// Loads a schema from directory structure (legacy method)
    #[deprecated(since = "1.0.0", note = "Use load_schema_internal() instead")]
    pub fn load_schema_from_directory(
        &self,
        domain: &str,
        category: &str,
        schema_name: &str,
    ) -> Result<Value> {
        let domain_path = Path::new(&self.schema_root).join(domain);

        // Determine version
        let version = if !self.version.is_empty() {
            self.version.clone()
        } else {
            if !domain_path.exists() || !domain_path.is_dir() {
                return Err(anyhow::anyhow!(
                    "Domain directory not found: {:?}",
                    domain_path
                ));
            }

            // Find the version directory (v{number})
            let version_regex = Regex::new(r"^v(\d+)$").unwrap();
            fs::read_dir(&domain_path)?
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
                .ok_or_else(|| {
                    anyhow::anyhow!("No version directory found in domain: {}", domain)
                })?
        };

        // Construct the full path: schemas/domain/v{number}/category/schemaName.json
        let schema_path = Path::new(&self.schema_root)
            .join(domain)
            .join(&version)
            .join(category)
            .join(format!("{}.json", schema_name));

        // Try filesystem first
        if schema_path.exists() {
            let schema_content = fs::read_to_string(schema_path)?;
            let schema: Value = serde_json::from_str(&schema_content)?;
            return Ok(schema);
        }

        // Fall back to embedded resource
        let embedded_path = format!("{}/{}/{}/{}.json", domain, version, category, schema_name);
        if let Some(file) = EmbeddedSchemas::get(&embedded_path) {
            let schema: Value = serde_json::from_slice(&file.data)?;
            return Ok(schema);
        }

        Err(anyhow::anyhow!(
            "Schema file not found in FS or embedded assets: {}",
            embedded_path
        ))
    }

    /// Loads a schema from a file (legacy method)
    #[deprecated(since = "1.0.0", note = "Use load_schema() instead")]
    pub fn load_schema_from_file(&self, schema_id: &str, version: &str) -> Result<Value> {
        let file_name = format!("{}_{}.json", schema_id, version);
        let schema_path = Path::new(&self.schema_root).join(&file_name);

        // Try filesystem
        if schema_path.exists() {
            let schema_content = fs::read_to_string(schema_path)?;
            let schema: Value = serde_json::from_str(&schema_content)?;
            return Ok(schema);
        }

        // Fall back to embedded
        if let Some(file) = EmbeddedSchemas::get(&file_name) {
            let schema: Value = serde_json::from_slice(&file.data)?;
            return Ok(schema);
        }

        Err(anyhow::anyhow!(
            "Schema file not found in FS or embedded assets: {}",
            file_name
        ))
    }

    // Legacy getters
    #[deprecated(since = "1.0.0", note = "Use get_schema_root() instead")]
    pub fn schema_base_path(&self) -> &str {
        &self.schema_root
    }

    #[deprecated(since = "1.0.0", note = "Use get_version() instead")]
    pub fn version_directory(&self) -> Option<&str> {
        if self.version.is_empty() {
            None
        } else {
            Some(&self.version)
        }
    }

    #[deprecated(since = "1.0.0", note = "Use constructor instead")]
    pub fn set_schema_base_path(&mut self, schema_base_path: String) {
        self.schema_root = schema_base_path;
    }

    #[deprecated(since = "1.0.0", note = "Use constructor instead")]
    pub fn set_version_directory(&mut self, version_directory: Option<String>) {
        self.version = version_directory.unwrap_or_default();
    }
}
