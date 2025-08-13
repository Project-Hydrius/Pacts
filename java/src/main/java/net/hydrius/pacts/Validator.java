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

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Validator class that validates data against schemas
 */
public class Validator {

    private final ObjectMapper objectMapper;
    private final SchemaLoader schemaLoader;

    public Validator() {
        this.objectMapper = new ObjectMapper();
        this.schemaLoader = new SchemaLoader();
    }

    public Validator(SchemaLoader schemaLoader) {
        this.objectMapper = new ObjectMapper();
        this.schemaLoader = schemaLoader;
    }

    /**
     * Validates an envelope against its schema
     */
    public ValidationResult validate(Envelope envelope) {
        List<String> errors = new ArrayList<>();

        try {
            // Validate header
            if (envelope.getHeader() == null) {
                errors.add("Header is required");
                return new ValidationResult(false, errors);
            }

            Header header = envelope.getHeader();
            if (header.getSchemaId() == null || header.getSchemaId().isEmpty()) {
                errors.add("Schema ID is required in header");
            }

            if (header.getSchemaVersion() == null || header.getSchemaVersion().isEmpty()) {
                errors.add("Schema version is required in header");
            }

            // Load and validate schema
            if (header.getSchemaId() != null && !header.getSchemaId().isEmpty()) {
                JsonNode schema = schemaLoader.loadSchema(header.getSchemaId(), header.getSchemaVersion());
                if (schema == null) {
                    errors.add("Schema not found: " + header.getSchemaId() + " version " + header.getSchemaVersion());
                } else {
                    // Basic schema validation
                    ValidationResult dataValidation = validateData(envelope.getData(), schema);
                    errors.addAll(dataValidation.getErrors());
                }
            }

        } catch (Exception e) {
            errors.add("Validation error: " + e.getMessage());
        }

        return new ValidationResult(errors.isEmpty(), errors);
    }

    /**
     * Validates data against a schema
     */
    public ValidationResult validateData(Object data, JsonNode schema) {
        List<String> errors = new ArrayList<>();

        try {
            JsonNode dataNode = objectMapper.valueToTree(data);

            // Basic required field validation
            if (schema.has("required")) {
                JsonNode requiredFields = schema.get("required");
                for (JsonNode field : requiredFields) {
                    String fieldName = field.asText();
                    if (!dataNode.has(fieldName)) {
                        errors.add("Required field missing: " + fieldName);
                    }
                }
            }

            // Basic type validation
            if (schema.has("type")) {
                String expectedType = schema.get("type").asText();
                if (!validateType(dataNode, expectedType)) {
                    errors.add("Invalid type. Expected: " + expectedType);
                }
            }

            // Property type validation
            if (schema.has("properties") && dataNode.isObject()) {
                JsonNode properties = schema.get("properties");
                Iterator<String> fieldNames = properties.fieldNames();
                while (fieldNames.hasNext()) {
                    String propertyName = fieldNames.next();
                    if (dataNode.has(propertyName)) {
                        JsonNode propertySchema = properties.get(propertyName);
                        if (propertySchema.has("type")) {
                            String propertyType = propertySchema.get("type").asText();
                            if (!validateType(dataNode.get(propertyName), propertyType)) {
                                errors.add("Invalid type for field '" + propertyName + "'. Expected: " + propertyType);
                            }
                        }
                    }
                }
            }

        } catch (Exception e) {
            errors.add("Data validation error: " + e.getMessage());
        }

        return new ValidationResult(errors.isEmpty(), errors);
    }

    private boolean validateType(JsonNode data, String expectedType) {
        return switch (expectedType) {
            case "object" ->
                data.isObject();
            case "array" ->
                data.isArray();
            case "string" ->
                data.isTextual();
            case "number" ->
                data.isNumber();
            case "boolean" ->
                data.isBoolean();
            case "null" ->
                data.isNull();
            default ->
                true;
        };
    }

}
