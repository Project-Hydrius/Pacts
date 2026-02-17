use anyhow::Result;
use log::{error, info, warn};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;

use zip::read::ZipArchive;

#[derive(Deserialize)]
struct SourcesConfig {
    sources: Vec<String>,
}

const CONNECTION_TIMEOUT_SECS: u64 = 30;
const MAX_RESPONSE_SIZE: u64 = 50 * 1024 * 1024;

/// Loads schemas from remote ZIP files.
#[derive(Clone)]
pub struct SchemaLoader {
    schema_cache: HashMap<String, Value>,
    schema_root: String,
    domain: String,
    version: String,
}

impl SchemaLoader {
    /// Creates a new SchemaLoader with the specified schema root, domain, and version.
    pub fn new(schema_root: String, domain: String, version: String) -> Self {
        if schema_root.is_empty() || domain.is_empty() || version.is_empty() {
            panic!("Schema root, domain, and version must be specified.");
        }

        let mut loader = Self {
            schema_cache: HashMap::new(),
            schema_root,
            domain,
            version,
        };

        info!(
            "Initializing SchemaLoader with root: {}, domain: {}, version: {}",
            loader.schema_root, loader.domain, loader.version
        );

        if let Err(e) = loader.load_remote_schemas() {
            error!("Failed to load remote schemas: {}", e);
            panic!("Failed to load remote schemas: {}", e);
        }

        info!(
            "SchemaLoader initialized successfully with {} schemas in cache",
            loader.schema_cache.len()
        );
        loader
    }

    /// Loads a schema from cache by category and name.
    pub fn load_schema(&mut self, category: &str, name: &str) -> Value {
        let cache_key = format!("{}/{}/{}/{}", self.domain, self.version, category, name);

        if let Some(schema) = self.schema_cache.get(&cache_key) {
            return schema.clone();
        }

        panic!(
            "Schema not found in cache: {}/{}/{}/{}",
            self.domain, self.version, category, name
        );
    }

    /// Clears all cached schemas.
    pub fn clear_cache(&mut self) {
        self.schema_cache.clear();
    }

    /// Returns the schema root directory.
    pub fn get_schema_root(&self) -> &str {
        &self.schema_root
    }

    /// Returns the domain.
    pub fn get_domain(&self) -> &str {
        &self.domain
    }

    /// Returns the version string.
    pub fn get_version(&self) -> &str {
        &self.version
    }

    /// Returns the parsed version as an integer.
    pub fn get_parsed_version(&self) -> i32 {
        self.version.replace("v", "").parse().unwrap_or(1)
    }

    fn load_remote_schemas(&mut self) -> Result<()> {
        let sources = self.load_sources_config()?;

        for source in sources {
            let cache_size_before = self.schema_cache.len();
            match self.load_schemas_from_zip_url(&source) {
                Ok(_) => {
                    if self.schema_cache.len() > cache_size_before {
                        info!("Successfully loaded schemas from: {}", source);
                        return Ok(());
                    }
                    warn!(
                        "ZIP from {} contained no loadable schemas, trying next source",
                        source
                    );
                }
                Err(e) => {
                    error!("Failed to load schemas from {}: {}", source, e);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Sources could not be read or found to populate schemas."
        ))
    }

    fn load_sources_config(&self) -> Result<Vec<String>> {
        const SOURCES_YAML: &str = include_str!(concat!(env!("OUT_DIR"), "/sources.yaml"));

        let config: SourcesConfig = serde_yaml::from_str(SOURCES_YAML)
            .map_err(|e| anyhow::anyhow!("Failed to parse embedded sources.yaml: {}", e))?;

        if config.sources.is_empty() {
            return Err(anyhow::anyhow!("No sources defined in sources.yaml"));
        }

        Ok(config.sources)
    }

    fn load_schemas_from_zip_url(&mut self, url: &str) -> Result<()> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(
                CONNECTION_TIMEOUT_SECS,
            )))
            .build()
            .into();

        let mut response = agent
            .get(url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP request to {} failed: {}", url, e))?;

        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_RESPONSE_SIZE)
            .read_to_end(&mut bytes)?;

        let reader = std::io::Cursor::new(bytes);
        let mut zip = ZipArchive::new(reader)?;

        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;

            if !entry.is_dir() && entry.name().ends_with(".json") {
                let entry_name = entry.name().to_string();

                let mut content = String::new();
                if let Err(e) = entry.read_to_string(&mut content) {
                    error!("Failed to read entry {} (index {}): {}", entry_name, i, e);
                    continue;
                }

                let schema: Value = match serde_json::from_str(&content) {
                    Ok(s) => s,
                    Err(e) => {
                        error!(
                            "Failed to parse JSON for entry {} (index {}): {}",
                            entry_name, i, e
                        );
                        continue;
                    }
                };

                let entry_path = entry_name.as_str();
                let last_slash = entry_path.rfind('/');
                let (category_path, file_name) = match last_slash {
                    Some(pos) => (&entry_path[..pos], &entry_path[pos + 1..]),
                    None => ("", entry_path),
                };

                let path_parts: Vec<&str> = category_path.split('/').collect();
                if path_parts.len() >= 3 {
                    let entry_domain = path_parts[path_parts.len() - 3];
                    let entry_version = path_parts[path_parts.len() - 2];
                    let entry_category = path_parts[path_parts.len() - 1];
                    let schema_name = file_name.trim_end_matches(".json");

                    let cache_key = format!(
                        "{}/{}/{}/{}",
                        entry_domain, entry_version, entry_category, schema_name
                    );
                    self.schema_cache.insert(cache_key.clone(), schema);
                    info!("Loaded schema into cache: {}", cache_key);
                }
            }
        }

        Ok(())
    }
}
