package net.hydrius.pacts;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class EnvelopeTest {

    private Header testHeader;
    private Map<String, Object> testData;
    private Map<String, Object> testMetadata;

    @BeforeEach
    void setup() {
        testHeader = new Header("1.0", "test-schema");
        testData = new HashMap<>();
        testData.put("id", "123");
        testData.put("name", "Test Item");

        testMetadata = new HashMap<>();
        testMetadata.put("source", "test");
        testMetadata.put("priority", "high");
    }

    @Test
    void testDefaultConstructor() {
        Envelope envelope = new Envelope();

        assertNull(envelope.getHeader());
        assertNull(envelope.getData());
        assertNull(envelope.getMetadata());
    }

    @Test
    void testConstructorWithHeaderAndData() {
        Envelope envelope = new Envelope(testHeader, testData);

        assertEquals(testHeader, envelope.getHeader());
        assertEquals(testData, envelope.getData());
        assertNull(envelope.getMetadata());
    }

    @Test
    void testConstructorWithHeaderDataAndMetadata() {
        Envelope envelope = new Envelope(testHeader, testData, testMetadata);

        assertEquals(testHeader, envelope.getHeader());
        assertEquals(testData, envelope.getData());
        assertEquals(testMetadata, envelope.getMetadata());
    }

    @Test
    void testSetAndGetHeader() {
        Envelope envelope = new Envelope();
        envelope.setHeader(testHeader);

        assertEquals(testHeader, envelope.getHeader());
    }

    @Test
    void testSetAndGetData() {
        Envelope envelope = new Envelope();
        envelope.setData(testData);

        assertEquals(testData, envelope.getData());
    }

    @Test
    void testSetAndGetMetadata() {
        Envelope envelope = new Envelope();
        envelope.setMetadata(testMetadata);

        assertEquals(testMetadata, envelope.getMetadata());
    }

    @Test
    void testSetNullValues() {
        Envelope envelope = new Envelope(testHeader, testData, testMetadata);

        envelope.setHeader(null);
        envelope.setData(null);
        envelope.setMetadata(null);

        assertNull(envelope.getHeader());
        assertNull(envelope.getData());
        assertNull(envelope.getMetadata());
    }

    @Test
    void testWithPrimitiveData() {
        Envelope envelope = new Envelope(testHeader, "simple string data");

        assertEquals(testHeader, envelope.getHeader());
        assertEquals("simple string data", envelope.getData());
    }

    @Test
    void testWithNullData() {
        Envelope envelope = new Envelope(testHeader, null);

        assertEquals(testHeader, envelope.getHeader());
        assertNull(envelope.getData());
    }

    @Test
    void testWithEmptyMetadata() {
        Map<String, Object> emptyMetadata = new HashMap<>();
        Envelope envelope = new Envelope(testHeader, testData, emptyMetadata);

        assertEquals(testHeader, envelope.getHeader());
        assertEquals(testData, envelope.getData());
        assertEquals(emptyMetadata, envelope.getMetadata());
    }
}
