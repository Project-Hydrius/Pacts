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
package net.hydrius.pacts.model;

import java.time.Instant;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Header class that contains metadata about the envelope
 */
public class Header {

    @JsonProperty("schema_version")
    private String schemaVersion;

    @JsonProperty("schema_category")
    private String schemaCategory;

    @JsonProperty("schema_name")
    private String schemaName;

    @JsonProperty("timestamp")
    private Instant timestamp;

    @JsonProperty("content_type")
    private String contentType;

    public Header() {
    }

    /**
     * Creates a new header with the given schema version, category, and name
     *
     * @param schemaVersion the version of the schema
     * @param schemaCategory the category of the schema
     * @param schemaName the name of the schema
     */
    public Header(String schemaVersion, String schemaCategory, String schemaName) {
        this.schemaVersion = schemaVersion;
        this.schemaCategory = schemaCategory;
        this.schemaName = schemaName;
        this.timestamp = Instant.now();
    }

    /**
     * Creates a new header with the given schema version, category, name, and
     * content type
     *
     * @param schemaVersion the version of the schema
     * @param schemaCategory the category of the schema
     * @param schemaName the name of the schema
     * @param contentType the content type of the schema
     */
    public Header(String schemaVersion, String schemaCategory, String schemaName, String contentType) {
        this.schemaVersion = schemaVersion;
        this.schemaCategory = schemaCategory;
        this.schemaName = schemaName;
        this.contentType = contentType;
        this.timestamp = Instant.now();
    }

    /**
     * Gets the schema version
     *
     * @return the schema version
     */
    public String getSchemaVersion() {
        return schemaVersion;
    }

    /**
     * Sets the schema version
     *
     * @param schemaVersion the schema version
     */
    public void setSchemaVersion(String schemaVersion) {
        this.schemaVersion = schemaVersion;
    }

    /**
     * Gets the schema category
     *
     * @return the schema category
     */
    public String getSchemaCategory() {
        return schemaCategory;
    }

    /**
     * Sets the schema category
     *
     * @param schemaCategory the schema category
     */
    public void setSchemaCategory(String schemaCategory) {
        this.schemaCategory = schemaCategory;
    }

    /**
     * Gets the schema name
     *
     * @return the schema name
     */
    public String getSchemaName() {
        return schemaName;
    }

    /**
     * Sets the schema name
     *
     * @param schemaName the schema name
     */
    public void setSchemaName(String schemaName) {
        this.schemaName = schemaName;
    }

    /**
     * Gets the timestamp
     *
     * @return the timestamp
     */
    public Instant getTimestamp() {
        return timestamp;
    }

    /**
     * Sets the timestamp
     *
     * @param timestamp the timestamp
     */
    public void setTimestamp(Instant timestamp) {
        this.timestamp = timestamp;
    }

    /**
     * Gets the content type
     *
     * @return the content type
     */
    public String getContentType() {
        return contentType;
    }

    /**
     * Sets the content type
     *
     * @param contentType the content type
     */
    public void setContentType(String contentType) {
        this.contentType = contentType;
    }

}
