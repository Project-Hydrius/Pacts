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

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Service class for convenient Pacts operations
 */
public class PactsService {

    private final Validator validator;
    private final SchemaLoader schemaLoader;
    private final ObjectMapper objectMapper;

    public PactsService() {
        this.schemaLoader = new SchemaLoader();
        this.validator = new Validator(schemaLoader);
        this.objectMapper = new ObjectMapper();
    }

    public PactsService(String schemaBasePath) {
        this.schemaLoader = new SchemaLoader(schemaBasePath);
        this.validator = new Validator(schemaLoader);
        this.objectMapper = new ObjectMapper();
    }

    public PactsService(SchemaLoader schemaLoader) {
        this.schemaLoader = schemaLoader;
        this.validator = new Validator(schemaLoader);
        this.objectMapper = new ObjectMapper();
    }

    /**
     * Creates an envelope with authentication
     */
    public Envelope createEnvelope(String schemaVersion, String schemaId, Object data, String authToken) {
        Header header = new Header(schemaVersion, schemaId, "application/json", authToken);
        return new Envelope(header, data);
    }

    /**
     * Creates an envelope without authentication
     */
    public Envelope createEnvelope(String schemaVersion, String schemaId, Object data) {
        Header header = new Header(schemaVersion, schemaId, "application/json");
        return new Envelope(header, data);
    }

    /**
     * Validates an envelope and returns the result
     */
    public ValidationResult validate(Envelope envelope) {
        return validator.validate(envelope);
    }

    /**
     * Validates data against a specific schema
     */
    public ValidationResult validateData(Object data, String domain, String category, String schemaName) {
        JsonNode schema = schemaLoader.loadSchemaByDirectory(domain, category, schemaName);
        if (schema == null) {
            ValidationResult result = new ValidationResult();
            result.setValid(false);
            result.setErrors(java.util.List.of("Schema not found: " + domain + "/" + category + "/" + schemaName));
            return result;
        }

        return validator.validateData(data, schema);
    }

    /**
     * Convenience method to send validated data
     */
    public <T> T sendValidatedData(String schemaVersion, String schemaId, Object data,
            String authToken, MessageSender<T> sender) throws Exception {
        Envelope envelope = createEnvelope(schemaVersion, schemaId, data, authToken);
        ValidationResult result = validate(envelope);

        if (!result.isValid()) {
            throw new ValidationException("Validation failed: " + result.getErrorMessage());
        }

        return sender.send(envelope);
    }

    /**
     * Parses JSON string to an envelope
     */
    public Envelope parseEnvelope(String json) throws Exception {
        return objectMapper.readValue(json, Envelope.class);
    }

    /**
     * Converts envelope to JSON string
     */
    public String toJson(Envelope envelope) throws Exception {
        return objectMapper.writeValueAsString(envelope);
    }

    /**
     * Gets the validator
     */
    public Validator getValidator() {
        return validator;
    }

    /**
     * Gets the schema loader
     */
    public SchemaLoader getSchemaLoader() {
        return schemaLoader;
    }

    /**
     * Functional interface for message sending
     */
    @FunctionalInterface
    public interface MessageSender<T> {

        T send(Envelope envelope) throws Exception;
    }

    /**
     * Custom exception for validation errors
     */
    public static class ValidationException extends Exception {

        public ValidationException(String message) {
            super(message);
        }
    }
}
