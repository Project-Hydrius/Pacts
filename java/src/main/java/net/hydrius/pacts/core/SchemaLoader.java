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
package net.hydrius.pacts.core;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * SchemaLoader class that loads schemas that are bundled with Pacts.
 */
public class SchemaLoader {

    private final ObjectMapper objectMapper;
    private final Map<String, JsonNode> cache;

    private final String schemaRoot;
    private final String domain;
    private final String version;

    /**
     * Creates a new SchemaLoader.
     *
     * @param schemaRoot the directory containing the schemas
     * @param domain the domain of the schema
     * @param version the version of the schema
     * @throws IllegalArgumentException if the schema root, domain, or version
     * is null
     */
    public SchemaLoader(String schemaRoot, String domain, String version) throws IllegalArgumentException {
        if (schemaRoot == null || domain == null || version == null) {
            throw new IllegalArgumentException("Schema root, domain, and version must be specified.");
        }

        this.objectMapper = new ObjectMapper();
        this.cache = new HashMap<>();
        this.schemaRoot = schemaRoot;
        this.domain = domain;
        this.version = version;
    }

    /**
     * Loads a schema from cache, file system, or classpath by category and
     * name.
     *
     * @param category The schema category (e.g., "player")
     * @param name The schema name (e.g., "player_request")
     * @return The parsed JSON schema node
     * @throws IOException if the schema is not found
     */
    public JsonNode loadSchema(String category, String name) throws IOException {
        String cacheKey = domain + "/" + version + "/" + category + "/" + name;

        if (cache.containsKey(cacheKey)) {
            return cache.get(cacheKey);
        }

        try {
            JsonNode schema = loadSchemaInternal(category, name);
            if (schema != null) {
                cache.put(cacheKey, schema);
            }
            return schema;
        } catch (IOException e) {
            throw new IOException("Failed to load schema: " + cacheKey, e);
        }
    }

    /**
     * Attempts to load schema from file system, then classpath.
     *
     * @param category the category of the schema
     * @param name the name of the schema
     * @return the schema
     * @throws IOException if the schema is not found
     */
    private JsonNode loadSchemaInternal(String category, String name) throws IOException {
        Path filePath = Paths.get(schemaRoot, domain, version, category, name + ".json");

        if (Files.exists(filePath)) {
            return objectMapper.readTree(Files.readString(filePath));
        }

        String resourcePath = schemaRoot + "/" + domain + "/" + version + "/" + category + "/" + name + ".json";
        return loadSchemaFromResource(resourcePath);
    }

    /**
     * Loads a schema from the classpath (inside the JAR).
     *
     * @param resourcePath Path relative to classpath root (e.g.,
     * "schemas/bees/v1/player/player_request.json")
     * @return the schema
     * @throws IOException if the schema is not found
     */
    private JsonNode loadSchemaFromResource(String resourcePath) throws IOException {
        try (InputStream stream = getClass().getClassLoader().getResourceAsStream(resourcePath)) {
            if (stream == null) {
                throw new IOException("Resource not found: " + resourcePath);
            }
            return objectMapper.readTree(stream);
        }
    }

    /**
     * Loads a schema from a raw string.
     *
     * @param schemaContent the schema content
     * @return the schema
     * @throws IOException if the schema is not found
     */
    public JsonNode loadSchemaFromString(String schemaContent) throws IOException {
        return objectMapper.readTree(schemaContent);
    }

    /**
     * Loads a schema from a raw input stream.
     *
     * @param inputStream the input stream
     * @return the schema
     * @throws IOException if the schema is not found
     */
    public JsonNode loadSchemaFromStream(InputStream inputStream) throws IOException {
        return objectMapper.readTree(inputStream);
    }

    /**
     * Clears all cached schemas.
     */
    public void clearCache() {
        cache.clear();
    }

    /**
     * Gets the schema root.
     *
     * @return the schema root
     */
    public String getSchemaRoot() {
        return schemaRoot;
    }

    /**
     * Gets the domain.
     *
     * @return the domain
     */
    public String getDomain() {
        return domain;
    }

    /**
     * Gets the version.
     *
     * @return the version
     */
    public String getVersion() {
        return version;
    }

    /**
     * Gets the parsed version.
     *
     * @return the parsed version
     */
    public int getParsedVersion() {
        return Integer.parseInt(version.replace("v", ""));
    }
}
