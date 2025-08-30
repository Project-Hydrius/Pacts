package net.hydrius.pacts.core;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import net.hydrius.pacts.model.Envelope;
import net.hydrius.pacts.model.Header;

/**
 * Validator class that validates data against schemas
 */
public class Validator {

    private final ObjectMapper objectMapper;
    private final SchemaLoader schemaLoader;

    /**
     * Creates a new validator with a schema loader
     *
     * @param schemaLoader the schema loader to use
     */
    public Validator(SchemaLoader schemaLoader) {
        this.objectMapper = new ObjectMapper();
        this.schemaLoader = schemaLoader;
    }

    /**
     * Validates an envelope against its schema
     *
     * @param envelope the envelope to validate
     * @return the validation result
     */
    public ValidationResult validate(Envelope envelope) {
        List<String> errors = new ArrayList<>();

        try {
            if (envelope.getHeader() == null) {
                errors.add("Header is required");
                return new ValidationResult(false, errors);
            }

            Header header = envelope.getHeader();
            if (header.getSchemaCategory() == null || header.getSchemaCategory().isEmpty()) {
                errors.add("Schema category is required in header");
            }

            if (header.getSchemaName() == null || header.getSchemaName().isEmpty()) {
                errors.add("Schema name is required in header");
            }

            if (header.getSchemaVersion() == null || header.getSchemaVersion().isEmpty()) {
                errors.add("Schema version is required in header");
            }

            if (header.getSchemaCategory() != null
                    && !header.getSchemaCategory().isEmpty()
                    && header.getSchemaName() != null
                    && !header.getSchemaName().isEmpty()) {

                JsonNode schema = schemaLoader.loadSchema(header.getSchemaCategory(), header.getSchemaName());
                if (schema == null) {
                    errors.add("Schema not found: " + header.getSchemaCategory() + "/" + header.getSchemaName());
                } else {
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
     * Validates the type of a value
     *
     * @param data the data to validate
     * @param expectedType the expected type
     * @return true if the type is valid, false otherwise
     */
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

    /**
     * Validates the required fields of a schema
     *
     * @param dataNode the data to validate
     * @param schema the schema to validate against
     * @param errors the list of errors
     */
    private void validateRequiredFields(JsonNode dataNode, JsonNode schema, List<String> errors) {
        if (schema.has("required")) {
            JsonNode requiredFields = schema.get("required");
            for (JsonNode field : requiredFields) {
                String fieldName = field.asText();
                if (!dataNode.has(fieldName)) {
                    errors.add("Required field missing: " + fieldName);
                }
            }
        }
    }

    /**
     * Validates the type of a value
     *
     * @param dataNode the data to validate
     * @param schema the schema to validate against
     * @param errors the list of errors
     */
    private void validateType(JsonNode dataNode, JsonNode schema, List<String> errors) {
        if (schema.has("type")) {
            String expectedType = schema.get("type").asText();
            if (!validateType(dataNode, expectedType)) {
                errors.add("Invalid type. Expected: " + expectedType);
            }
        }
    }

    /**
     * Validates the type of a property
     *
     * @param dataNode the data to validate
     * @param propertyName the name of the property
     * @param propertySchema the schema to validate against
     * @param errors the list of errors
     */
    private void validatePropertyType(JsonNode dataNode, String propertyName, JsonNode propertySchema, List<String> errors) {
        if (propertySchema.has("type")) {
            String propertyType = propertySchema.get("type").asText();
            if (!validateType(dataNode.get(propertyName), propertyType)) {
                errors.add("Invalid type for field '" + propertyName + "'. Expected: " + propertyType);
            }
        }
    }

    /**
     * Validates the properties of a schema
     *
     * @param dataNode the data to validate
     * @param schema the schema to validate against
     * @param errors the list of errors
     */
    private void validateProperties(JsonNode dataNode, JsonNode schema, List<String> errors) {
        if (schema.has("properties") && dataNode.isObject()) {
            JsonNode properties = schema.get("properties");
            Iterator<String> fieldNames = properties.fieldNames();
            while (fieldNames.hasNext()) {
                String propertyName = fieldNames.next();
                if (dataNode.has(propertyName)) {
                    JsonNode propertySchema = properties.get(propertyName);
                    validatePropertyType(dataNode, propertyName, propertySchema, errors);
                }
            }
        }
    }

    /**
     * Validates data against a schema
     *
     * @param data the data to validate
     * @param schema the schema to validate against
     * @return the validation result
     */
    public ValidationResult validateData(Object data, JsonNode schema) {
        List<String> errors = new ArrayList<>();

        JsonNode dataNode = objectMapper.valueToTree(data);

        validateRequiredFields(dataNode, schema, errors);
        validateType(dataNode, schema, errors);
        validateProperties(dataNode, schema, errors);

        return new ValidationResult(errors.isEmpty(), errors);
    }

}
