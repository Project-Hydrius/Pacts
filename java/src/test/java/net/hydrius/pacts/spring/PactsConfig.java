package net.hydrius.pacts.spring;

import java.io.IOException;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import tools.jackson.databind.json.JsonMapper;

import net.hydrius.pacts.core.SchemaLoader;
import net.hydrius.pacts.core.Validator;
import net.hydrius.pacts.impl.PactsService;

@Configuration
public class PactsConfig {

    @Bean
    public JsonMapper jsonMapper() {
        return JsonMapper.builder().build();
    }

    @Bean
    public SchemaLoader schemaLoader() throws IOException {
        return new SchemaLoader("bees", "v1");
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
