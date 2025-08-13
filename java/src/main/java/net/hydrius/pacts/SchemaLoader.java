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
package net.hydrius.pacts;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;
import java.util.regex.Pattern;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * SchemaLoader class that loads schemas from various sources
 */
public class SchemaLoader {

    private final ObjectMapper objectMapper;
    private final Map<String, JsonNode> schemaCache;
    private String schemaBasePath;
    private static final Pattern VERSION_PATTERN = Pattern.compile("v(\\d+)");

    public SchemaLoader() {
        this.objectMapper = new ObjectMapper();
        this.schemaCache = new HashMap<>();
        this.schemaBasePath = "schemas";
    }

    public SchemaLoader(String schemaBasePath) {
        this.objectMapper = new ObjectMapper();
        this.schemaCache = new HashMap<>();
        this.schemaBasePath = schemaBasePath;
    }

    /**
     * Loads a schema by ID and version
     */
    public JsonNode loadSchema(String schemaId, String version) {
        String cacheKey = schemaId + "_" + version;

        if (schemaCache.containsKey(cacheKey)) {
            return schemaCache.get(cacheKey);
        }

        try {
            JsonNode schema = loadSchemaFromFile(schemaId, version);
            if (schema != null) {
                schemaCache.put(cacheKey, schema);
            }
            return schema;
        } catch (Exception e) {
            return null;
        }
    }

    /**
     * Loads a schema by directory structure: domain/category/schemaName The
     * version is automatically extracted from the v{number} directory
     */
    public JsonNode loadSchemaByDirectory(String domain, String category, String schemaName) {
        String cacheKey = domain + "_" + category + "_" + schemaName;

        if (schemaCache.containsKey(cacheKey)) {
            return schemaCache.get(cacheKey);
        }

        try {
            JsonNode schema = loadSchemaFromDirectory(domain, category, schemaName);
            if (schema != null) {
                schemaCache.put(cacheKey, schema);
            }
            return schema;
        } catch (Exception e) {
            return null;
        }
    }

    /**
     * Loads a schema from directory structure
     */
    public JsonNode loadSchemaFromDirectory(String domain, String category, String schemaName) throws IOException {
        Path domainPath = Paths.get(schemaBasePath, domain);

        if (!Files.exists(domainPath) || !Files.isDirectory(domainPath)) {
            return null;
        }

        // Find the version directory (v{number})
        String version = null;
        try {
            version = Files.list(domainPath)
                    .filter(Files::isDirectory)
                    .map(Path::getFileName)
                    .map(Path::toString)
                    .filter(name -> VERSION_PATTERN.matcher(name).matches())
                    .findFirst()
                    .orElse(null);
        } catch (IOException e) {
            return null;
        }

        if (version == null) {
            return null;
        }

        // Construct the full path: schemas/domain/v{number}/category/schemaName.json
        Path schemaPath = Paths.get(schemaBasePath, domain, version, category, schemaName + ".json");

        if (!Files.exists(schemaPath)) {
            // Try loading from resources
            String resourcePath = domain + "/" + version + "/" + category + "/" + schemaName + ".json";
            return loadSchemaFromResource(resourcePath);
        }

        String schemaContent = Files.readString(schemaPath);
        return objectMapper.readTree(schemaContent);
    }

    /**
     * Loads a schema from a file
     */
    public JsonNode loadSchemaFromFile(String schemaId, String version) throws IOException {
        String fileName = schemaId + "_" + version + ".json";
        Path schemaPath = Paths.get(schemaBasePath, fileName);

        if (!Files.exists(schemaPath)) {
            // Try loading from resources
            return loadSchemaFromResource(fileName);
        }

        String schemaContent = Files.readString(schemaPath);
        return objectMapper.readTree(schemaContent);
    }

    /**
     * Loads a schema from an input stream
     */
    public JsonNode loadSchemaFromStream(InputStream inputStream) throws IOException {
        return objectMapper.readTree(inputStream);
    }

    /**
     * Loads a schema from a string
     */
    public JsonNode loadSchemaFromString(String schemaContent) throws IOException {
        return objectMapper.readTree(schemaContent);
    }

    /**
     * Loads a schema from resources (classpath)
     */
    public JsonNode loadSchemaFromResource(String resourcePath) throws IOException {
        String fullPath = "/" + schemaBasePath + "/" + resourcePath;
        InputStream stream = getClass().getResourceAsStream(fullPath);

        if (stream == null) {
            // Try without leading slash
            fullPath = schemaBasePath + "/" + resourcePath;
            stream = getClass().getResourceAsStream(fullPath);
        }

        if (stream == null) {
            return null;
        }

        try (InputStream inputStream = stream) {
            return objectMapper.readTree(inputStream);
        }
    }

    /**
     * Clears the schema cache
     */
    public void clearCache() {
        schemaCache.clear();
    }

    /**
     * Sets the base path for schema files
     */
    public void setSchemaBasePath(String schemaBasePath) {
        this.schemaBasePath = schemaBasePath;
    }

    /**
     * Gets the base path for schema files
     */
    public String getSchemaBasePath() {
        return schemaBasePath;
    }
}
