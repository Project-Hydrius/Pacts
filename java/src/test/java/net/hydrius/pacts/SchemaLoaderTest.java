package net.hydrius.pacts;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;
import org.junit.jupiter.api.io.TempDir;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class SchemaLoaderTest {

    private SchemaLoader schemaLoader;
    private ObjectMapper objectMapper;
    private Path tempSchemasDir;

    @BeforeEach
    void setup(@TempDir Path tempDir) throws IOException {
        schemaLoader = new SchemaLoader(tempDir.toString());
        objectMapper = new ObjectMapper();
        tempSchemasDir = tempDir;

        // Create test schema directory structure
        createTestSchemaStructure();
    }

    private void createTestSchemaStructure() throws IOException {
        // Create domain directory
        Path domainPath = tempSchemasDir.resolve("test-domain");
        Files.createDirectories(domainPath);

        // Create version directory
        Path versionPath = domainPath.resolve("v1");
        Files.createDirectories(versionPath);

        // Create category directory
        Path categoryPath = versionPath.resolve("test-category");
        Files.createDirectories(categoryPath);

        // Create test schema file
        String schemaContent = """
            {
                "type": "object",
                "required": ["id", "name"],
                "properties": {
                    "id": {"type": "string"},
                    "name": {"type": "string"},
                    "description": {"type": "string"}
                }
            }
            """;
        Files.write(categoryPath.resolve("test_schema.json"), schemaContent.getBytes());

        // Create another schema file
        String schema2Content = """
            {
                "type": "object",
                "required": ["code"],
                "properties": {
                    "code": {"type": "number"},
                    "status": {"type": "string"}
                }
            }
            """;
        Files.write(categoryPath.resolve("another_schema.json"), schema2Content.getBytes());
    }

    @Test
    void testDefaultConstructor() {
        SchemaLoader loader = new SchemaLoader();
        assertEquals("schemas", loader.getSchemaBasePath());
    }

    @Test
    void testConstructorWithBasePath() {
        String customPath = "/custom/schemas/path";
        SchemaLoader loader = new SchemaLoader(customPath);
        assertEquals(customPath, loader.getSchemaBasePath());
    }

    @Test
    void testLoadSchemaByDirectoryWithValidSchema() {
        JsonNode schema = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "test_schema");

        assertNotNull(schema);
        assertTrue(schema.isObject());
        assertEquals("object", schema.get("type").asText());
        assertTrue(schema.has("required"));
        assertTrue(schema.has("properties"));
    }

    @Test
    void testLoadSchemaByDirectoryWithNonExistentSchema() {
        JsonNode schema = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "non_existent");

        assertNull(schema);
    }

    @Test
    void testLoadSchemaByDirectoryWithNonExistentDomain() {
        JsonNode schema = schemaLoader.loadSchemaByDirectory("non_existent", "test-category", "test_schema");

        assertNull(schema);
    }

    @Test
    void testLoadSchemaByDirectoryWithNonExistentCategory() {
        JsonNode schema = schemaLoader.loadSchemaByDirectory("test-domain", "non_existent", "test_schema");

        assertNull(schema);
    }

    @Test
    void testLoadSchemaByDirectoryCaching() {
        // Load schema twice
        JsonNode schema1 = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "test_schema");
        JsonNode schema2 = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "test_schema");

        assertNotNull(schema1);
        assertNotNull(schema2);
        assertEquals(schema1, schema2);
    }

    @Test
    void testLoadSchemaByDirectoryWithMultipleSchemas() {
        JsonNode schema1 = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "test_schema");
        JsonNode schema2 = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "another_schema");

        assertNotNull(schema1);
        assertNotNull(schema2);
        assertNotEquals(schema1, schema2);

        // Verify different schemas have different structures
        assertTrue(schema1.get("required").toString().contains("id"));
        assertTrue(schema2.get("required").toString().contains("code"));
    }

    @Test
    void testLoadSchemaFromDirectoryWithValidPath() throws IOException {
        JsonNode schema = schemaLoader.loadSchemaFromDirectory("test-domain", "test-category", "test_schema");

        assertNotNull(schema);
        assertTrue(schema.isObject());
        assertEquals("object", schema.get("type").asText());
    }

    @Test
    void testLoadSchemaFromDirectoryWithNonExistentPath() throws IOException {
        JsonNode schema = schemaLoader.loadSchemaFromDirectory("non_existent", "test-category", "test_schema");
        assertNull(schema);
    }

    @Test
    void testLoadSchemaFromFile() throws IOException {
        // Create a test schema file
        String schemaContent = """
            {
                "type": "object",
                "properties": {
                    "test": {"type": "string"}
                }
            }
            """;
        Path schemaFile = tempSchemasDir.resolve("test_schema_1.0.json");
        Files.write(schemaFile, schemaContent.getBytes());

        JsonNode schema = schemaLoader.loadSchemaFromFile("test_schema", "1.0");

        assertNotNull(schema);
        assertTrue(schema.isObject());
        assertEquals("object", schema.get("type").asText());
    }

    @Test
    void testLoadSchemaFromFileWithNonExistentFile() throws IOException {
        JsonNode schema = schemaLoader.loadSchemaFromFile("non_existent", "1.0");
        assertNull(schema);
    }

    @Test
    void testLoadSchemaFromStream() throws IOException {
        String schemaContent = """
            {
                "type": "object",
                "properties": {
                    "stream_test": {"type": "number"}
                }
            }
            """;

        try (InputStream inputStream = new java.io.ByteArrayInputStream(schemaContent.getBytes())) {
            JsonNode schema = schemaLoader.loadSchemaFromStream(inputStream);

            assertNotNull(schema);
            assertTrue(schema.isObject());
            assertEquals("object", schema.get("type").asText());
        }
    }

    @Test
    void testLoadSchemaFromString() throws IOException {
        String schemaContent = """
            {
                "type": "object",
                "properties": {
                    "string_test": {"type": "boolean"}
                }
            }
            """;

        JsonNode schema = schemaLoader.loadSchemaFromString(schemaContent);

        assertNotNull(schema);
        assertTrue(schema.isObject());
        assertEquals("object", schema.get("type").asText());
    }

    @Test
    void testLoadSchemaFromStringWithInvalidJson() {
        assertThrows(IOException.class, () -> {
            schemaLoader.loadSchemaFromString("invalid json content");
        });
    }

    @Test
    void testClearCache() {
        // Load a schema to populate cache
        JsonNode schema1 = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "test_schema");
        assertNotNull(schema1);

        // Clear cache
        schemaLoader.clearCache();

        // Load again - should still work but from file
        JsonNode schema2 = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "test_schema");
        assertNotNull(schema2);
        assertEquals(schema1, schema2);
    }

    @Test
    void testSetAndGetSchemaBasePath() {
        String newPath = "/new/schema/path";
        schemaLoader.setSchemaBasePath(newPath);

        assertEquals(newPath, schemaLoader.getSchemaBasePath());
    }

    @Test
    void testLoadSchemaWithVersionAndId() {
        // Create a test schema file with version and ID naming
        try {
            String schemaContent = """
                {
                    "type": "object",
                    "properties": {
                        "version_test": {"type": "string"}
                    }
                }
                """;
            Path schemaFile = tempSchemasDir.resolve("test_id_1.0.json");
            Files.write(schemaFile, schemaContent.getBytes());

            JsonNode schema = schemaLoader.loadSchema("test_id", "1.0");

            assertNotNull(schema);
            assertTrue(schema.isObject());
            assertEquals("object", schema.get("type").asText());
        } catch (IOException e) {
            fail("Failed to create test schema file: " + e.getMessage());
        }
    }

    @Test
    void testLoadSchemaWithNonExistentVersionAndId() {
        JsonNode schema = schemaLoader.loadSchema("non_existent", "1.0");
        assertNull(schema);
    }

    @Test
    void testLoadSchemaCaching() {
        // Create a test schema file
        try {
            String schemaContent = """
                {
                    "type": "object",
                    "properties": {
                        "cache_test": {"type": "string"}
                    }
                }
                """;
            Path schemaFile = tempSchemasDir.resolve("cache_test_1.0.json");
            Files.write(schemaFile, schemaContent.getBytes());

            // Load schema twice
            JsonNode schema1 = schemaLoader.loadSchema("cache_test", "1.0");
            JsonNode schema2 = schemaLoader.loadSchema("cache_test", "1.0");

            assertNotNull(schema1);
            assertNotNull(schema2);
            assertEquals(schema1, schema2);
        } catch (IOException e) {
            fail("Failed to create test schema file: " + e.getMessage());
        }
    }

    @Test
    void testWithInvalidSchemaContent() throws IOException {
        // Create a schema file with invalid JSON
        Path schemaFile = tempSchemasDir.resolve("test-domain").resolve("v1").resolve("test-category").resolve("invalid_schema.json");
        Files.write(schemaFile, "invalid json content".getBytes());

        JsonNode schema = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "invalid_schema");

        // Should return null for invalid JSON
        assertNull(schema);
    }

    @Test
    void testWithEmptySchemaFile() throws IOException {
        // Create an empty schema file
        Path schemaFile = tempSchemasDir.resolve("test-domain").resolve("v1").resolve("test-category").resolve("empty_schema.json");
        Files.write(schemaFile, "".getBytes());

        JsonNode schema = schemaLoader.loadSchemaByDirectory("test-domain", "test-category", "empty_schema");

        // Should return null for empty file (invalid JSON)
        assertTrue(schema.isEmpty());
    }
}
