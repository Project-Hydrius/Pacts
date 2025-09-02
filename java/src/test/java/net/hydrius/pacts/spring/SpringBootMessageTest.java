package net.hydrius.pacts.spring;

import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.util.Date;
import java.util.UUID;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.amqp.core.Message;
import org.springframework.amqp.core.MessageProperties;
import org.springframework.amqp.rabbit.core.RabbitTemplate;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.testcontainers.junit.jupiter.Testcontainers;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import net.hydrius.pacts.core.ValidationResult;
import net.hydrius.pacts.impl.PactsService;
import net.hydrius.pacts.model.Envelope;

@Testcontainers
@SpringBootTest(classes = {
    PactsConfig.class,
    RabbitConfig.class,
    TestListener.class
})
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
public class SpringBootMessageTest {

    private static final Logger logger = LoggerFactory.getLogger(SpringBootMessageTest.class);
    public static final String EXCHANGE = "test.request.exchange";
    public static final String ROUTING_KEY = "test.request.key";
    public static final String QUEUE = "test.request.queue";

    @Autowired
    private PactsService pactsService;

    @Autowired
    private RabbitTemplate rabbitTemplate;

    @Autowired
    private ObjectMapper objectMapper;

    void sendMessage(JsonNode json, String schemaCategory, String schemaName) throws JsonProcessingException {
        logger.debug("Preparing to send message for schema: {}/{}", schemaCategory, schemaName);

        Envelope envelope = pactsService.createEnvelope(schemaCategory, schemaName, json);
        String payload = pactsService.toJson(envelope);

        logger.debug("Created envelope with payload size: {} bytes", payload.length());

        MessageProperties props = new MessageProperties();
        props.setContentType("application/json");
        props.setTimestamp(Date.from(Instant.now()));

        ValidationResult result = pactsService.validate(envelope);
        if (!result.isValid()) {
            logger.error("Validation failed for {}/{}: {}", schemaCategory, schemaName, result.getErrorMessage());
            throw new IllegalArgumentException("Invalid payload. Error: " + result.getErrorMessage());
        }

        logger.debug("Validation successful for {}/{}", schemaCategory, schemaName);

        Message message = new Message(payload.getBytes(StandardCharsets.UTF_8), props);
        rabbitTemplate.convertAndSend(EXCHANGE, ROUTING_KEY, message);
        logger.info("Successfully sent message to exchange '{}' with routing key '{}' - Schema: {}/{}",
                EXCHANGE, ROUTING_KEY, schemaCategory, schemaName);
    }

    void sendMessageAndExpectFailure(JsonNode json, String schemaCategory, String schemaName, String expectedErrorSubstring) {
        logger.debug("Testing expected validation failure for schema: {}/{}", schemaCategory, schemaName);

        try {
            sendMessage(json, schemaCategory, schemaName);
            Assertions.fail("Expected validation to fail but it succeeded");
        } catch (IllegalArgumentException e) {
            logger.debug("Validation failed as expected: {}", e.getMessage());
            if (expectedErrorSubstring != null) {
                Assertions.assertTrue(e.getMessage().contains(expectedErrorSubstring),
                        "Error message should contain: " + expectedErrorSubstring + ", but was: " + e.getMessage());
            }
        } catch (JsonProcessingException e) {
            logger.error("JSON processing error during validation test", e);
            Assertions.fail("Expected IllegalArgumentException but got JsonProcessingException: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("Valid player request should process successfully")
    void goodPayload_processesEnvelopeAndValidates() throws Exception {
        logger.info("Testing valid player request payload");

        var payload = objectMapper.createObjectNode()
                .put("request_type", "PLAYER_JOIN")
                .put("target_id", UUID.randomUUID().toString())
                .put("date", Instant.now().toString());

        sendMessage(payload, "player", "player_request");
        logger.info("Valid player request test completed successfully");
    }

    @Test
    @DisplayName("Player request missing date should throw validation error")
    void missingDate_throwsHelpfulError() {
        logger.info("Testing player request with missing date field");

        var payload = objectMapper.createObjectNode()
                .put("request_type", "PLAYER_JOIN")
                .put("target_id", UUID.randomUUID().toString());

        sendMessageAndExpectFailure(payload, "player", "player_request", "date");
        logger.info("Missing date validation test completed successfully");
    }

    @Test
    @DisplayName("Player request missing target_id should fail validation")
    void missingTargetId_throwsValidationError() {
        logger.info("Testing player request with missing target_id field");

        var payload = objectMapper.createObjectNode()
                .put("request_type", "PLAYER_LEAVE")
                .put("date", Instant.now().toString());

        sendMessageAndExpectFailure(payload, "player", "player_request", "target_id");
        logger.info("Missing target_id validation test completed successfully");
    }

    @Test
    @DisplayName("Player request missing request_type should fail validation")
    void missingRequestType_throwsValidationError() {
        logger.info("Testing player request with missing request_type field");

        var payload = objectMapper.createObjectNode()
                .put("target_id", UUID.randomUUID().toString())
                .put("date", Instant.now().toString());

        sendMessageAndExpectFailure(payload, "player", "player_request", "request_type");
        logger.info("Missing request_type validation test completed successfully");
    }

    @Test
    @DisplayName("Valid basic inventory item should validate successfully")
    void validBasicInventoryItem_shouldValidate() {
        logger.info("Testing valid basic inventory item");

        var payload = objectMapper.createObjectNode()
                .put("slot", 1)
                .put("material", "diamond_sword")
                .put("amount", 1);

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_item", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertTrue(result.isValid(), "Basic inventory item should be valid: " + result.getErrorMessage());
        logger.info("Basic inventory item validation test completed successfully");
    }

    @Test
    @DisplayName("Inventory item with enchantments should validate successfully")
    void inventoryItemWithEnchantments_shouldValidate() {
        logger.info("Testing inventory item with enchantments");

        ArrayNode enchantments = objectMapper.createArrayNode();
        ObjectNode enchantment = objectMapper.createObjectNode()
                .put("enchantment_id", "sharpness")
                .put("enchantment_level", 5);
        enchantments.add(enchantment);

        var payload = objectMapper.createObjectNode()
                .put("slot", 2)
                .put("material", "diamond_sword")
                .put("amount", 1)
                .set("enchantment_data", enchantments);

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_item", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertTrue(result.isValid(), "Inventory item with enchantments should be valid: " + result.getErrorMessage());
        logger.info("Inventory item with enchantments validation test completed successfully");
    }

    @Test
    @DisplayName("Inventory item with complex NBT data should validate successfully")
    void inventoryItemWithComplexNBT_shouldValidate() {
        logger.info("Testing inventory item with complex NBT data");

        // Create nectar statistics
        ArrayNode statistics = objectMapper.createArrayNode();
        ObjectNode stat = objectMapper.createObjectNode()
                .put("type", "attack_damage")
                .put("value", 10.5)
                .put("boosts", 3);
        statistics.add(stat);

        // Create nectar data
        ArrayNode nectar = objectMapper.createArrayNode();
        ObjectNode nectarSlot = objectMapper.createObjectNode()
                .put("slot", 1)
                .set("statistics", statistics);
        nectar.add(nectarSlot);

        // Create NBT data
        ObjectNode nbtData = objectMapper.createObjectNode()
                .put("id", "unique_sword_123")
                .put("rarity", "legendary")
                .put("level", 50)
                .put("modifier", "fire_aspect")
                .set("nectar", nectar);

        var payload = objectMapper.createObjectNode()
                .put("slot", 3)
                .put("material", "legendary_sword")
                .put("amount", 1)
                .set("nbt_data", nbtData);

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_item", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertTrue(result.isValid(), "Complex inventory item should be valid: " + result.getErrorMessage());
        logger.info("Complex inventory item validation test completed successfully");
    }

    @Test
    @DisplayName("Inventory item missing required fields should fail validation")
    void inventoryItemMissingRequiredFields_shouldFail() {
        logger.info("Testing inventory item with missing required fields");

        var payload = objectMapper.createObjectNode()
                .put("slot", 1); // Missing material and amount

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_item", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertFalse(result.isValid(), "Inventory item with missing fields should be invalid");
        logger.debug("Validation failed as expected: {}", result.getErrorMessage());
        logger.info("Missing required fields validation test completed successfully");
    }

    @Test
    @DisplayName("Inventory request with inventory array should validate successfully")
    void inventoryRequestWithInventoryArray_shouldValidate() {
        logger.info("Testing inventory request with inventory array");

        ArrayNode inventory = objectMapper.createArrayNode();
        ObjectNode item = objectMapper.createObjectNode()
                .put("slot", 1)
                .put("material", "diamond")
                .put("amount", 64);
        inventory.add(item);

        var payload = objectMapper.createObjectNode()
                .put("date", Instant.now().toString())
                .put("request_type", "GET_INVENTORY")
                .put("target_id", UUID.randomUUID().toString())
                .put("profile_id", UUID.randomUUID().toString())
                .set("inventory", inventory);

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_request", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertTrue(result.isValid(), "Inventory request with inventory array should be valid: " + result.getErrorMessage());
        logger.info("Inventory request with inventory array validation test completed successfully");
    }

    @Test
    @DisplayName("Inventory request with slot should validate successfully")
    void inventoryRequestWithSlot_shouldValidate() {
        logger.info("Testing inventory request with slot");

        var payload = objectMapper.createObjectNode()
                .put("date", Instant.now().toString())
                .put("request_type", "GET_SLOT")
                .put("target_id", UUID.randomUUID().toString())
                .put("profile_id", UUID.randomUUID().toString())
                .put("slot", 5);

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_request", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertTrue(result.isValid(), "Inventory request with slot should be valid: " + result.getErrorMessage());
        logger.info("Inventory request with slot validation test completed successfully");
    }

    @Test
    @DisplayName("Inventory request missing profile_id should fail validation")
    void inventoryRequestMissingProfileId_shouldFail() {
        logger.info("Testing inventory request with missing profile_id");

        var payload = objectMapper.createObjectNode()
                .put("date", Instant.now().toString())
                .put("request_type", "GET_INVENTORY")
                .put("target_id", UUID.randomUUID().toString())
                .put("slot", 1);

        Envelope envelope = pactsService.createEnvelope("inventory", "inventory_request", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertFalse(result.isValid(), "Inventory request without profile_id should be invalid");
        logger.debug("Validation failed as expected: {}", result.getErrorMessage());
        logger.info("Missing profile_id validation test completed successfully");
    }

    @Test
    @DisplayName("Valid profile request should validate successfully")
    void validProfileRequest_shouldValidate() {
        logger.info("Testing valid profile request");

        var payload = objectMapper.createObjectNode()
                .put("date", Instant.now().toString())
                .put("request_type", "GET_PROFILE")
                .put("target_id", UUID.randomUUID().toString());

        Envelope envelope = pactsService.createEnvelope("profile", "profile_request", payload);
        ValidationResult result = pactsService.validate(envelope);

        Assertions.assertTrue(result.isValid(), "Profile request should be valid: " + result.getErrorMessage());
        logger.info("Profile request validation test completed successfully");
    }

    @Test
    @DisplayName("Null payload should be handled gracefully")
    void nullPayload_shouldBeHandledGracefully() {
        logger.info("Testing null payload handling");

        Exception exception = Assertions.assertThrows(Exception.class, () -> {
            sendMessage(null, "player", "player_request");
        });
        logger.debug("Null payload correctly threw exception: {}", exception.getMessage());

        logger.info("Null payload handling test completed successfully");
    }

    @Test
    @DisplayName("Empty payload should fail validation appropriately")
    void emptyPayload_shouldFailValidation() {
        logger.info("Testing empty payload validation");

        var payload = objectMapper.createObjectNode();
        sendMessageAndExpectFailure(payload, "player", "player_request", null);

        logger.info("Empty payload validation test completed successfully");
    }

    @Test
    @DisplayName("Multiple rapid validations should perform adequately")
    void multipleRapidValidations_shouldPerformAdequately() {
        logger.info("Testing performance with multiple rapid validations");

        long startTime = System.currentTimeMillis();

        for (int i = 0; i < 100; i++) {
            var payload = objectMapper.createObjectNode()
                    .put("request_type", "PERFORMANCE_TEST_" + i)
                    .put("target_id", UUID.randomUUID().toString())
                    .put("date", Instant.now().toString());

            Envelope envelope = pactsService.createEnvelope("player", "player_request", payload);
            ValidationResult result = pactsService.validate(envelope);
            Assertions.assertTrue(result.isValid(), "Validation " + i + " should succeed");
        }

        long endTime = System.currentTimeMillis();
        long duration = endTime - startTime;

        logger.info("Completed 100 validations in {} ms (avg: {} ms per validation)",
                duration, duration / 100.0);

        // Performance assertion - should complete 100 validations in reasonable time
        Assertions.assertTrue(duration < 5000,
                "100 validations should complete in under 5 seconds, took: " + duration + "ms");
    }

}
