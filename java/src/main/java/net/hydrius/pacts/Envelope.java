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

import java.util.Map;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Envelope class that wraps data with metadata for schema validation
 */
public class Envelope {

    @JsonProperty("header")
    private Header header;

    @JsonProperty("data")
    private Object data;

    @JsonProperty("metadata")
    private Map<String, Object> metadata;

    public Envelope() {
    }

    public Envelope(Header header, Object data) {
        this.header = header;
        this.data = data;
    }

    public Envelope(Header header, Object data, Map<String, Object> metadata) {
        this.header = header;
        this.data = data;
        this.metadata = metadata;
    }

    public Header getHeader() {
        return header;
    }

    public void setHeader(Header header) {
        this.header = header;
    }

    public Object getData() {
        return data;
    }

    public void setData(Object data) {
        this.data = data;
    }

    public Map<String, Object> getMetadata() {
        return metadata;
    }

    public void setMetadata(Map<String, Object> metadata) {
        this.metadata = metadata;
    }
}
