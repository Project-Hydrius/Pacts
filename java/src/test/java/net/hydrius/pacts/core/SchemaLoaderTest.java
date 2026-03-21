package net.hydrius.pacts.core;

import static org.junit.jupiter.api.Assertions.*;

import java.io.IOException;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class SchemaLoaderTest {

    private static SchemaLoader schemaLoader;

    @BeforeAll
    static void setup() throws IOException {
        schemaLoader = new SchemaLoader("bees", "v1");
    }

    @Test
    void testSchemaLoaderInitialization() {
        assertNotNull(schemaLoader);
        assertEquals("bees", schemaLoader.getDomain());
        assertEquals("v1", schemaLoader.getVersion());
    }

}
