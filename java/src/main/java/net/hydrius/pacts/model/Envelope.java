package net.hydrius.pacts.model;

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

    /**
     * Creates a new envelope with header and data
     *
     * @param header the header of the envelope
     * @param data the data of the envelope
     */
    public Envelope(Header header, Object data) {
        this.header = header;
        this.data = data;
    }

    /**
     * Creates a new envelope with header, data, and metadata
     *
     * @param header the header of the envelope
     * @param data the data of the envelope
     * @param metadata the metadata of the envelope
     */
    public Envelope(Header header, Object data, Map<String, Object> metadata) {
        this.header = header;
        this.data = data;
        this.metadata = metadata;
    }

    /**
     * Gets the header of the envelope
     *
     * @return the header of the envelope
     */
    public Header getHeader() {
        return header;
    }

    /**
     * Sets the header of the envelope
     *
     * @param header the header of the envelope
     */
    public void setHeader(Header header) {
        this.header = header;
    }

    /**
     * Gets the data of the envelope
     *
     * @return the data of the envelope
     */
    public Object getData() {
        return data;
    }

    /**
     * Sets the data of the envelope
     *
     * @param data the data of the envelope
     */
    public void setData(Object data) {
        this.data = data;
    }

    /**
     * Gets the metadata of the envelope
     *
     * @return the metadata of the envelope
     */
    public Map<String, Object> getMetadata() {
        return metadata;
    }

    /**
     * Sets the metadata of the envelope
     *
     * @param metadata the metadata of the envelope
     */
    public void setMetadata(Map<String, Object> metadata) {
        this.metadata = metadata;
    }
}
