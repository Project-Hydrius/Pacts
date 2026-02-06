package net.hydrius.pacts.impl;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.IOException;
import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertFalse;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

import net.hydrius.pacts.core.SchemaLoader;
import net.hydrius.pacts.core.ValidationResult;
import net.hydrius.pacts.model.Envelope;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class PactsServiceTest {

    private PactsService pactsService;

    @BeforeEach
    void setup() throws IOException {
        SchemaLoader schemaLoader = new SchemaLoader("schemas", "bees", "v1");
        pactsService = new PactsService(schemaLoader);
    }

    @Test
    void testCreateEnvelope() {
        Map<String, Object> playerData = new HashMap<>();
        playerData.put("target_id", "player-123");
        playerData.put("request_type", "PLAYER_JOIN");
        playerData.put("date", "2025-01-01");

        Envelope envelope = pactsService.createEnvelope("player", "player_request", playerData);
        assertEquals("v1", envelope.getHeader().getSchemaVersion());
        assertEquals("player", envelope.getHeader().getSchemaCategory());
        assertEquals("player_request", envelope.getHeader().getSchemaName());
        assertEquals(playerData, envelope.getData());
    }

    @Test
    void testValidateEnvelopeWithNullHeader() {
        Envelope envelope = new Envelope(null, null);
        ValidationResult result = pactsService.validate(envelope);
        assertFalse(result.isValid());
        assertEquals(1, result.getErrors().size());
        assertEquals("Header is required", result.getErrors().get(0));
    }

}
