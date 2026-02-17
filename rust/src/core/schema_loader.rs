use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;

use zip::read::ZipArchive;

#[derive(Deserialize)]
struct SourcesConfig {
    sources: Vec<String>,
}

/// SchemaLoader struct that loads schemas from remote sources.
/// 
/// This class loads schemas exclusively from remote ZIP files.
/// No local file system or embedded resources are used.
#[derive(Clone)]
pub struct SchemaLoader {
    schema_cache: HashMap<String, Value>,
    schema_root: String,
    domain: String,
    version: String,
}

impl SchemaLoader {
    /// Creates a new SchemaLoader and loads schemas from remote sources.
    ///
    /// # Arguments
    /// * `schema_root` - reserved for future local file system loading (must be non-empty)
    /// * `domain` - the domain of the schema
    /// * `version` - the version of the schema
    /// 
    /// # Panics
    /// Panics if the schema root, domain, or version is empty, or if remote schemas cannot be loaded
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
        
        // Load remote schemas on initialization - must succeed
        loader.load_remote_schemas().expect("Failed to load remote schemas");
        
        loader
    }

    /// Loads a schema from cache by category and name.
    ///
    /// # Arguments
    /// * `category` - The schema category (e.g., "player")
    /// * `name` - The schema name (e.g., "player_request")
    ///
    /// # Returns
    /// The parsed JSON schema value
    ///
    /// # Panics
    /// Panics if the schema cannot be found in cache
    pub fn load_schema(&mut self, category: &str, name: &str) -> Value {
        let cache_key = format!("{}/{}/{}/{}", self.domain, self.version, category, name);

        // Only check cache - no fallbacks
        if let Some(schema) = self.schema_cache.get(&cache_key) {
            return schema.clone();
        }

        panic!(
            "Schema not found in cache: {}/{}/{}/{}",
            self.domain, self.version, category, name
        );
    }

    /// Loads schemas from remote ZIP files specified in sources.yaml.
    /// 
    /// This method processes ZIP files in memory without writing to disk,
    /// extracting all JSON schema files and storing them in an in-memory cache.
    /// 
    /// The loader supports multiple source URLs for redundancy, trying each one
    /// in order until a successful load occurs.
    /// 
    /// # Returns
    /// Result indicating success or failure
    fn load_remote_schemas(&mut self) -> Result<()> {
        // Read sources from configuration file
        let sources = self.load_sources_config()?;
        
        let mut sources_loaded = false;
        
        for source in sources {
            let cache_size_before = self.schema_cache.len();
            match self.load_schemas_from_zip_url(&source) {
                Ok(_) => {
                    if self.schema_cache.len() > cache_size_before {
                        eprintln!("Successfully loaded schemas from: {}", source);
                        sources_loaded = true;
                        break;
                    } else {
                        eprintln!("ZIP from {} contained no loadable schemas, trying next source", source);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load schemas from {}: {}", source, e);
                }
            }
        }
        
        if sources_loaded {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Sources could not be read or found to populate schemas."))
        }
    }
    
    /// Loads sources configuration embedded at compile time from sources.yaml.
    /// 
    /// # Returns
    /// Result containing vector of source URLs
    fn load_sources_config(&self) -> Result<Vec<String>> {
        const SOURCES_YAML: &str = include_str!(concat!(env!("OUT_DIR"), "/sources.yaml"));

        let config: SourcesConfig = serde_yaml::from_str(SOURCES_YAML)
            .map_err(|e| anyhow::anyhow!("Failed to parse embedded sources.yaml: {}", e))?;

        if config.sources.is_empty() {
            return Err(anyhow::anyhow!("No sources defined in sources.yaml"));
        }

        Ok(config.sources)
    }
    
    /// Loads schemas from a ZIP file at the given URL.
    /// 
    /// # Arguments
    /// * `url` - The URL of the ZIP file to load schemas from
    /// 
    /// # Returns
    /// Result indicating success or failure
    fn load_schemas_from_zip_url(&mut self, url: &str) -> Result<()> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build()
            .into();

        let mut response = agent.get(url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP request to {} failed: {}", url, e))?;

        const MAX_RESPONSE_SIZE: u64 = 50 * 1024 * 1024; // 50 MB total
        let mut bytes = Vec::new();
        response.body_mut().as_reader().take(MAX_RESPONSE_SIZE).read_to_end(&mut bytes)?;
            
        // Process the ZIP archive in memory
        let reader = std::io::Cursor::new(bytes);
        let mut zip = ZipArchive::new(reader)?;
        
        // Iterate through all entries in the ZIP file
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            
            // Process only JSON files
            if !entry.is_dir() && entry.name().ends_with(".json") {
                let entry_name = entry.name().to_string();
                
                // Read the entry content
                let mut content = String::new();
                if let Err(e) = entry.read_to_string(&mut content) {
                    eprintln!("Failed to read entry {} (index {}): {}", entry_name, i, e);
                    continue;
                }
                
                // Parse the JSON schema
                let schema: Value = match serde_json::from_str(&content) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to parse JSON for entry {} (index {}): {}", entry_name, i, e);
                        continue;
                    }
                };
                
                // Extract path information to create cache key
                let path_parts: Vec<&str> = entry_name.split('/').collect();
                
                // We expect paths like: Schemas-main/bees/v1/inventory/item.json
                if path_parts.len() >= 5 {
                    let entry_domain = path_parts[path_parts.len() - 4];
                    let entry_version = path_parts[path_parts.len() - 3];
                    let entry_category = path_parts[path_parts.len() - 2];
                    let file_name = path_parts[path_parts.len() - 1];
                    
                    let schema_name = file_name.trim_end_matches(".json");
                    
                    let cache_key = format!("{}/{}/{}/{}", entry_domain, entry_version, entry_category, schema_name);
                    self.schema_cache.insert(cache_key.clone(), schema);
                    eprintln!("Loaded schema into cache: {}", cache_key);
                } else {
                    eprintln!("Skipping JSON file with unexpected path structure (< 5 components): {}", entry_name);
                }
            }
        }
        
        Ok(())
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
