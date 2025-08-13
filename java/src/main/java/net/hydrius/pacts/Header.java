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

import java.time.Instant;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Header class that contains metadata about the envelope
 */
public class Header {

    @JsonProperty("schema_version")
    private String schemaVersion;

    @JsonProperty("schema_id")
    private String schemaId;

    @JsonProperty("timestamp")
    private Instant timestamp;

    @JsonProperty("content_type")
    private String contentType;

    @JsonProperty("auth_token")
    private String authToken;

    public Header() {
    }

    public Header(String schemaVersion, String schemaId) {
        this.schemaVersion = schemaVersion;
        this.schemaId = schemaId;
        this.timestamp = Instant.now();
    }

    public Header(String schemaVersion, String schemaId, String contentType) {
        this.schemaVersion = schemaVersion;
        this.schemaId = schemaId;
        this.contentType = contentType;
        this.timestamp = Instant.now();
    }

    public Header(String schemaVersion, String schemaId, String contentType, String authToken) {
        this.schemaVersion = schemaVersion;
        this.schemaId = schemaId;
        this.contentType = contentType;
        this.authToken = authToken;
        this.timestamp = Instant.now();
    }

    public String getSchemaVersion() {
        return schemaVersion;
    }

    public void setSchemaVersion(String schemaVersion) {
        this.schemaVersion = schemaVersion;
    }

    public String getSchemaId() {
        return schemaId;
    }

    public void setSchemaId(String schemaId) {
        this.schemaId = schemaId;
    }

    public Instant getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(Instant timestamp) {
        this.timestamp = timestamp;
    }

    public String getContentType() {
        return contentType;
    }

    public void setContentType(String contentType) {
        this.contentType = contentType;
    }

    public String getAuthToken() {
        return authToken;
    }

    public void setAuthToken(String authToken) {
        this.authToken = authToken;
    }
}
