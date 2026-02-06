package net.hydrius.pacts.spring;

import java.io.IOException;

import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import com.fasterxml.jackson.databind.ObjectMapper;

import net.hydrius.pacts.core.SchemaLoader;
import net.hydrius.pacts.core.Validator;
import net.hydrius.pacts.impl.PactsService;

@Configuration
public class PactsConfig {

    @Bean
    public ObjectMapper objectMapper() {
        return new ObjectMapper().registerModule(new JavaTimeModule());
    }

    @Bean
    public SchemaLoader schemaLoader() throws IOException {
        return new SchemaLoader("schemas", "bees", "v1");
    }

    @Bean
    public Validator validator(SchemaLoader schemaLoader) {
        return new Validator(schemaLoader);
    }

    @Bean
    public PactsService pactsService(SchemaLoader schemaLoader) {
        return new PactsService(schemaLoader);
    }

}
