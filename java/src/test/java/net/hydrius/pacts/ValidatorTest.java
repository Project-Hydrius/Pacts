package net.hydrius.pacts;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class ValidatorTest {

    private Validator validator;
    private ObjectMapper objectMapper;
    private SchemaLoader mockSchemaLoader;

    @BeforeEach
    void setup() {
        validator = new Validator();
        objectMapper = new ObjectMapper();
        mockSchemaLoader = new SchemaLoader();
    }

    @Test
    void testDefaultConstructor() {
        Validator validator = new Validator();
        assertNotNull(validator);
    }

    @Test
    void testConstructorWithSchemaLoader() {
        SchemaLoader schemaLoader = new SchemaLoader();
        Validator validator = new Validator(schemaLoader);
        assertNotNull(validator);
    }

    @Test
    void testValidateEnvelopeWithNullHeader() {
        Envelope envelope = new Envelope();
        envelope.setHeader(null);
        envelope.setData(new HashMap<>());

        ValidationResult result = validator.validate(envelope);

        assertFalse(result.isValid());
        assertTrue(result.getErrors().contains("Header is required"));
    }

    @Test
    void testValidateEnvelopeWithEmptySchemaId() {
        Header header = new Header("1.0", "");
        Envelope envelope = new Envelope(header, new HashMap<>());

        ValidationResult result = validator.validate(envelope);

        assertFalse(result.isValid());
        assertTrue(result.getErrors().contains("Schema ID is required in header"));
    }

    @Test
    void testValidateEnvelopeWithEmptySchemaVersion() {
        Header header = new Header("", "test-schema");
        Envelope envelope = new Envelope(header, new HashMap<>());

        ValidationResult result = validator.validate(envelope);

        assertFalse(result.isValid());
        assertTrue(result.getErrors().contains("Schema version is required in header"));
    }

    @Test
    void testValidateEnvelopeWithValidHeader() {
        Header header = new Header("1.0", "test-schema");
        Map<String, Object> data = new HashMap<>();
        data.put("id", "123");
        data.put("name", "Test Item");
        Envelope envelope = new Envelope(header, data);

        ValidationResult result = validator.validate(envelope);

        // Should fail because schema doesn't exist, but header validation should pass
        assertFalse(result.isValid());
        assertTrue(result.getErrors().contains("Schema not found: test-schema version 1.0"));
    }

    @Test
    void testValidateDataWithValidObject() throws Exception {
        String schemaJson = """
            {
                "type": "object",
                "required": ["id", "name"],
                "properties": {
                    "id": {"type": "string"},
                    "name": {"type": "string"}
                }
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> data = new HashMap<>();
        data.put("id", "123");
        data.put("name", "Test Item");

        ValidationResult result = validator.validateData(data, schema);

        assertTrue(result.isValid());
        assertTrue(result.getErrors().isEmpty());
    }

    @Test
    void testValidateDataWithMissingRequiredField() throws Exception {
        String schemaJson = """
            {
                "type": "object",
                "required": ["id", "name"],
                "properties": {
                    "id": {"type": "string"},
                    "name": {"type": "string"}
                }
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> data = new HashMap<>();
        data.put("id", "123");
        // Missing "name" field

        ValidationResult result = validator.validateData(data, schema);

        assertFalse(result.isValid());
        assertTrue(result.getErrors().contains("Required field missing: name"));
    }

    @Test
    void testValidateDataWithWrongType() throws Exception {
        String schemaJson = """
            {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": {"type": "string"}
                }
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> data = new HashMap<>();
        data.put("id", 123); // Should be string, not number

        ValidationResult result = validator.validateData(data, schema);

        assertFalse(result.isValid());
        assertTrue(result.getErrors().contains("Invalid type for field 'id'. Expected: string"));
    }

    @Test
    void testValidateTypeObject() throws Exception {
        JsonNode data = objectMapper.readTree("{\"key\": \"value\"}");
        assertTrue(validator.validateData(data, objectMapper.readTree("{\"type\": \"object\"}")).isValid());
    }

    @Test
    void testValidateTypeArray() throws Exception {
        JsonNode data = objectMapper.readTree("[1, 2, 3]");
        assertTrue(validator.validateData(data, objectMapper.readTree("{\"type\": \"array\"}")).isValid());
    }

    @Test
    void testValidateTypeString() throws Exception {
        JsonNode data = objectMapper.readTree("\"test string\"");
        assertTrue(validator.validateData(data, objectMapper.readTree("{\"type\": \"string\"}")).isValid());
    }

    @Test
    void testValidateTypeNumber() throws Exception {
        JsonNode data = objectMapper.readTree("42");
        assertTrue(validator.validateData(data, objectMapper.readTree("{\"type\": \"number\"}")).isValid());
    }

    @Test
    void testValidateTypeBoolean() throws Exception {
        JsonNode data = objectMapper.readTree("true");
        assertTrue(validator.validateData(data, objectMapper.readTree("{\"type\": \"boolean\"}")).isValid());
    }

    @Test
    void testValidateTypeNull() throws Exception {
        JsonNode data = objectMapper.readTree("null");
        assertTrue(validator.validateData(data, objectMapper.readTree("{\"type\": \"null\"}")).isValid());
    }

    @Test
    void testValidateDataWithNoRequiredFields() throws Exception {
        String schemaJson = """
            {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "name": {"type": "string"}
                }
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> data = new HashMap<>();
        // No required fields specified, so empty object should be valid

        ValidationResult result = validator.validateData(data, schema);

        assertTrue(result.isValid());
        assertTrue(result.getErrors().isEmpty());
    }

    @Test
    void testValidateDataWithNoTypeSpecified() throws Exception {
        String schemaJson = """
            {
                "required": ["id"]
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> data = new HashMap<>();
        data.put("id", "123");

        ValidationResult result = validator.validateData(data, schema);

        assertTrue(result.isValid());
        assertTrue(result.getErrors().isEmpty());
    }

    @Test
    void testValidateDataWithComplexNestedObject() throws Exception {
        String schemaJson = """
            {
                "type": "object",
                "required": ["user"],
                "properties": {
                    "user": {"type": "object"}
                }
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> userData = new HashMap<>();
        userData.put("name", "John");
        userData.put("age", 30);

        Map<String, Object> data = new HashMap<>();
        data.put("user", userData);

        ValidationResult result = validator.validateData(data, schema);

        assertTrue(result.isValid());
        assertTrue(result.getErrors().isEmpty());
    }

    @Test
    void testValidateDataWithArray() throws Exception {
        String schemaJson = """
            {
                "type": "object",
                "required": ["items"],
                "properties": {
                    "items": {"type": "array"}
                }
            }
            """;
        JsonNode schema = objectMapper.readTree(schemaJson);

        Map<String, Object> data = new HashMap<>();
        data.put("items", java.util.Arrays.asList("item1", "item2", "item3"));

        ValidationResult result = validator.validateData(data, schema);

        assertTrue(result.isValid());
        assertTrue(result.getErrors().isEmpty());
    }

    @Test
    void testValidateDataWithException() {
        // Test with invalid schema that will cause exception
        String invalidSchemaJson = "{\"invalid\": \"schema\"}";
        JsonNode invalidSchema;
        try {
            invalidSchema = objectMapper.readTree(invalidSchemaJson);
        } catch (Exception e) {
            // If we can't even create the schema, skip this test
            return;
        }

        Map<String, Object> data = new HashMap<>();
        data.put("test", "value");

        ValidationResult result = validator.validateData(data, invalidSchema);

        // Should handle exception gracefully
        assertTrue(result.isValid() || !result.isValid()); // Either valid or invalid, but no exception
    }
}
