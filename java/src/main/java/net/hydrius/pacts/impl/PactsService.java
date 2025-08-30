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
package net.hydrius.pacts.impl;

import java.util.List;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import net.hydrius.pacts.core.SchemaLoader;
import net.hydrius.pacts.core.ValidationResult;
import net.hydrius.pacts.core.Validator;
import net.hydrius.pacts.model.Envelope;
import net.hydrius.pacts.model.Header;

import java.io.IOException;

/**
 * Service class for convenient Pacts operations
 */
public class PactsService {

    private final Validator validator;
    private final SchemaLoader schemaLoader;
    private final ObjectMapper objectMapper;

    public PactsService(SchemaLoader schemaLoader) {
        this.schemaLoader = schemaLoader;
        this.validator = new Validator(schemaLoader);
        this.objectMapper = new ObjectMapper();
    }

    /**
     * Creates an envelope without authentication
     *
     * @param category the category of the schema
     * @param name the name of the schema
     * @param data the data to send
     * @return the envelope
     */
    public Envelope createEnvelope(String category, String name, Object data) {
        Header header = new Header(schemaLoader.getVersion(), category, name, "application/json");
        return new Envelope(header, data);
    }

    /**
     * Validates an envelope and returns the result
     *
     * @param envelope the envelope to validate
     * @return the validation result
     */
    public ValidationResult validate(Envelope envelope) {
        return validator.validate(envelope);
    }

    /**
     * Validates data against a specific schema
     *
     * @param data the data to validate
     * @param category the category of the schema
     * @param name the name of the schema
     * @return the validation result
     * @throws IOException if the schema is not found
     */
    public ValidationResult validateData(Object data, String category, String name) throws IOException {
        JsonNode schema = schemaLoader.loadSchema(category, name);

        if (schema == null) {
            ValidationResult result = new ValidationResult();
            result.setValid(false);
            result.setErrors(List.of(
                    "Schema not found: "
                    + schemaLoader.getDomain() + "/"
                    + schemaLoader.getVersion() + "/"
                    + category + "/" + name
            ));
            return result;
        }

        return validator.validateData(data, schema);
    }

    /**
     * Sends validated data using a provided sender function.
     *
     * @param category the category of the schema
     * @param name the name of the schema
     * @param data the data to send
     * @param sender the sender function
     * @return the result of the sender function
     * @throws Exception if the validation fails
     */
    public <T> T sendValidatedData(
            String category,
            String name,
            Object data,
            MessageSender<T> sender
    ) throws Exception {
        Envelope envelope = createEnvelope(category, name, data);
        ValidationResult result = validate(envelope);

        if (!result.isValid()) {
            throw new ValidationException("Validation failed: " + result.getErrorMessage());
        }

        return sender.send(envelope);
    }

    /**
     * Parses JSON string to an envelope
     */
    public Envelope parseEnvelope(String json) throws JsonProcessingException {
        return objectMapper.readValue(json, Envelope.class);
    }

    /**
     * Converts envelope to JSON string
     */
    public String toJson(Envelope envelope) throws JsonProcessingException {
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
