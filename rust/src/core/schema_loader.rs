use anyhow::Result;
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

}
