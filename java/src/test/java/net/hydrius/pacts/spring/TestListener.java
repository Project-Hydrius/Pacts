package net.hydrius.pacts.spring;

import java.nio.charset.StandardCharsets;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.amqp.rabbit.annotation.RabbitListener;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

import static net.hydrius.pacts.spring.SpringBootMessageTest.QUEUE;
import net.hydrius.pacts.core.ValidationResult;
import net.hydrius.pacts.impl.PactsService;
import net.hydrius.pacts.model.Envelope;

@Component
public class TestListener {

    private final Logger logger = LoggerFactory.getLogger(TestListener.class);
    private final JsonMapper jsonMapper;
    private final PactsService pactsService;

    @Autowired
    public TestListener(JsonMapper jsonMapper, PactsService pactsService) {
        this.jsonMapper = jsonMapper;
        this.pactsService = pactsService;
    }

    @RabbitListener(queues = QUEUE)
    void handleMessage(byte[] raw) {
        try {
            String body = new String(raw, StandardCharsets.UTF_8);
            logger.debug("Received message: {} bytes", raw.length);
            logger.trace("Message content: {}", body);

            Envelope envelope = pactsService.parseEnvelope(body);
            ValidationResult result = pactsService.validate(envelope);

            if (!result.isValid()) {
                logger.error("Message validation failed: {}", result.getErrors());
                throw new IllegalArgumentException("Message validation failed: " + result.getErrors());
            }

            JsonNode data = jsonMapper.valueToTree(envelope.getData());
            String type = data.path("request_type").asText("UNKNOWN");
            String targetId = data.path("target_id").asText("UNKNOWN");

            logger.info("Successfully processed message - Type: {}, Target: {}, Schema: {}/{}",
                    type, targetId, envelope.getHeader().getSchemaCategory(), envelope.getHeader().getSchemaName());
        } catch (Exception e) {
            logger.error("Failed to process message: {}", e.getMessage(), e);
            throw new RuntimeException("Failed to process message: " + e.getMessage(), e);
        }
    }

}
