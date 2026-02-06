package net.hydrius.pacts.core;

import static org.junit.jupiter.api.Assertions.*;

import java.io.IOException;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class SchemaLoaderTest {

    private SchemaLoader schemaLoader;
    
    @BeforeEach
    void setup() throws IOException {
        schemaLoader = new SchemaLoader("schemas", "bees", "v1");
    }

    @Test
    void testSchemaLoaderInitialization() {
        assertNotNull(schemaLoader);
        assertEquals("schemas", schemaLoader.getSchemaRoot());
        assertEquals("bees", schemaLoader.getDomain());
        assertEquals("v1", schemaLoader.getVersion());
    }

}