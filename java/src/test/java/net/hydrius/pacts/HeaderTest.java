package net.hydrius.pacts;

import java.time.Instant;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class HeaderTest {

    private static final String TEST_VERSION = "1.0";
    private static final String TEST_SCHEMA_ID = "test-schema";
    private static final String TEST_CONTENT_TYPE = "application/json";

    @Test
    void testDefaultConstructor() {
        Header header = new Header();

        assertNull(header.getSchemaVersion());
        assertNull(header.getSchemaId());
        assertNull(header.getTimestamp());
        assertNull(header.getContentType());
    }

    @Test
    void testConstructorWithVersionAndId() {
        Header header = new Header(TEST_VERSION, TEST_SCHEMA_ID);

        assertEquals(TEST_VERSION, header.getSchemaVersion());
        assertEquals(TEST_SCHEMA_ID, header.getSchemaId());
        assertNotNull(header.getTimestamp());
        assertNull(header.getContentType());

        // Verify timestamp is recent (within last 5 seconds)
        Instant now = Instant.now();
        assertTrue(header.getTimestamp().isAfter(now.minusSeconds(5)));
        assertTrue(header.getTimestamp().isBefore(now.plusSeconds(5)));
    }

    @Test
    void testConstructorWithVersionIdAndContentType() {
        Header header = new Header(TEST_VERSION, TEST_SCHEMA_ID, TEST_CONTENT_TYPE);

        assertEquals(TEST_VERSION, header.getSchemaVersion());
        assertEquals(TEST_SCHEMA_ID, header.getSchemaId());
        assertEquals(TEST_CONTENT_TYPE, header.getContentType());
        assertNotNull(header.getTimestamp());

        // Verify timestamp is recent (within last 5 seconds)
        Instant now = Instant.now();
        assertTrue(header.getTimestamp().isAfter(now.minusSeconds(5)));
        assertTrue(header.getTimestamp().isBefore(now.plusSeconds(5)));
    }

    @Test
    void testSetAndGetSchemaVersion() {
        Header header = new Header();
        header.setSchemaVersion(TEST_VERSION);

        assertEquals(TEST_VERSION, header.getSchemaVersion());
    }

    @Test
    void testSetAndGetSchemaId() {
        Header header = new Header();
        header.setSchemaId(TEST_SCHEMA_ID);

        assertEquals(TEST_SCHEMA_ID, header.getSchemaId());
    }

    @Test
    void testSetAndGetTimestamp() {
        Header header = new Header();
        Instant testTimestamp = Instant.now();
        header.setTimestamp(testTimestamp);

        assertEquals(testTimestamp, header.getTimestamp());
    }

    @Test
    void testSetAndGetContentType() {
        Header header = new Header();
        header.setContentType(TEST_CONTENT_TYPE);

        assertEquals(TEST_CONTENT_TYPE, header.getContentType());
    }

    @Test
    void testSetNullValues() {
        Header header = new Header(TEST_VERSION, TEST_SCHEMA_ID, TEST_CONTENT_TYPE);

        header.setSchemaVersion(null);
        header.setSchemaId(null);
        header.setTimestamp(null);
        header.setContentType(null);

        assertNull(header.getSchemaVersion());
        assertNull(header.getSchemaId());
        assertNull(header.getTimestamp());
        assertNull(header.getContentType());
    }

    @Test
    void testEmptyStrings() {
        Header header = new Header("", "", "");

        assertEquals("", header.getSchemaVersion());
        assertEquals("", header.getSchemaId());
        assertEquals("", header.getContentType());
        assertNotNull(header.getTimestamp());
    }

    @Test
    void testTimestampConsistency() {
        Header header1 = new Header(TEST_VERSION, TEST_SCHEMA_ID);

        try { // Wait a bit for the timestamp to be different
            Thread.sleep(1);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }

        Header header2 = new Header(TEST_VERSION, TEST_SCHEMA_ID);

        // Both should have recent timestamps
        assertNotNull(header1.getTimestamp());
        assertNotNull(header2.getTimestamp());

        // Timestamps should be different (created at different times)
        assertNotEquals(header1.getTimestamp(), header2.getTimestamp());
    }

    @Test
    void testWithSpecialCharacters() {
        String specialVersion = "2.0-beta";
        String specialId = "schema-with-special-chars_123";
        String specialContentType = "application/xml; charset=utf-8";

        Header header = new Header(specialVersion, specialId, specialContentType);

        assertEquals(specialVersion, header.getSchemaVersion());
        assertEquals(specialId, header.getSchemaId());
        assertEquals(specialContentType, header.getContentType());
    }
}
