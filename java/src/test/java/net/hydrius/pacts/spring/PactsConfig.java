package net.hydrius.pacts.spring;

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
        return new ObjectMapper();
    }

    @Bean
    public SchemaLoader schemaLoader() {
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
