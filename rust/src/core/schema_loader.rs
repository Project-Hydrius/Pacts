use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, BufRead};
use std::process::Command;
use std::path::Path;
use std::fs::File;

use zip::read::ZipArchive;

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
    /// * `schema_root` - kept for API compatibility (not used)
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
            match self.load_schemas_from_zip_url(&source) {
                Ok(_) => {
                    eprintln!("Successfully loaded schemas from: {}", source);
                    sources_loaded = true;
                    break;
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
    
    /// Loads sources configuration from sources.yaml file.
    /// 
    /// Tries to load from rust/resources folder, then falls back to
    /// default configuration if the file is not found.
    /// 
    /// # Returns
    /// Result containing vector of source URLs
    fn load_sources_config(&self) -> Result<Vec<String>> {
        // Config file location
        let config_path = Path::new("rust/resources/sources.yaml");
        
        if config_path.exists() {
            eprintln!("Loading sources configuration from: {}", config_path.display());
            let file = File::open(&config_path)?;
            let reader = std::io::BufReader::new(file);
            let mut sources = Vec::new();
            let mut in_sources_section = false;
            
            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim();
                
                // Check if we're entering the sources section
                if trimmed == "sources:" {
                    in_sources_section = true;
                    continue;
                }
                
                // Parse source entries (lines starting with "- ")
                if in_sources_section && trimmed.starts_with("- ") {
                    let source = trimmed[2..].trim();
                    // Remove surrounding quotes if present
                    let source = source.trim_matches('"');
                    if !source.is_empty() {
                        sources.push(source.to_string());
                    }
                }
                // If we hit a non-indented line after sources section, we're done
                else if in_sources_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if !line.starts_with(' ') && !line.starts_with('\t') {
                        break;
                    }
                }
            }
            
            if !sources.is_empty() {
                return Ok(sources);
            }
        }
        
        // Fall back to default configuration
        eprintln!("Warning: Could not read sources.yaml, using default configuration");
        Ok(vec![
            "https://github.com/Project-Hydrius/Schemas/archive/refs/heads/main.zip".to_string()
        ])
    }
    
    /// Loads schemas from a ZIP file at the given URL.
    /// 
    /// # Arguments
    /// * `url` - The URL of the ZIP file to load schemas from
    /// 
    /// # Returns
    /// Result indicating success or failure
    fn load_schemas_from_zip_url(&mut self, url: &str) -> Result<()> {
        // Download the ZIP file using curl
        let output = Command::new("curl")
            .args(&["-L", "-s", "--fail", "--max-time", "30", url])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to download ZIP from {}: {}",
                url,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Process the ZIP archive in memory
        let bytes = output.stdout;
        let reader = std::io::Cursor::new(bytes);
        let mut zip = ZipArchive::new(reader)?;
        
        // Iterate through all entries in the ZIP file
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            
            // Process only JSON files
            if !entry.is_dir() && entry.name().ends_with(".json") {
                // Read the entry content
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                
                // Parse the JSON schema
                let schema: Value = serde_json::from_str(&content)?;
                
                // Extract path information to create cache key
                let entry_path = entry.name();
                let path_parts: Vec<&str> = entry_path.split('/').collect();
                
                // We expect paths like: Schemas-main/bees/v1/inventory/item.json
                if path_parts.len() >= 5 {
                    let entry_domain = path_parts[path_parts.len() - 4];  // e.g., "bees"
                    let entry_version = path_parts[path_parts.len() - 3]; // e.g., "v1"
                    let entry_category = path_parts[path_parts.len() - 2]; // e.g., "inventory"
                    let file_name = path_parts[path_parts.len() - 1];     // e.g., "item.json"
                    
                    // Remove .json extension from filename
                    let schema_name = file_name.trim_end_matches(".json");
                    
                    // Store in cache with proper key format
                    let cache_key = format!("{}/{}/{}/{}", entry_domain, entry_version, entry_category, schema_name);
                    self.schema_cache.insert(cache_key.clone(), schema);
                    eprintln!("Loaded schema into cache: {}", cache_key);
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
