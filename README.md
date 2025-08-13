# Pacts - Schema Validation for Microservices

A comprehensive schema validation system for JSON payload communication between Game Servers and Microservices, implemented in both Java and Rust. Designed for seamless integration with Spring Boot/RabbitMQ (Java) and Actix/RabbitMQ (Rust).

## Project Structure

```
pacts/
├── java/                    # Java implementation
│   ├── pom.xml             # Maven configuration
│   └── src/main/java/net/hydrius/pacts/
│       ├── Envelope.java   # Wraps data with metadata
│       ├── Header.java     # Contains envelope metadata
│       ├── Validator.java  # Validates data against schemas
│       ├── ValidationResult.java # Holds validation results
│       └── SchemaLoader.java # Loads schemas from various sources
├── rust/                   # Rust implementation
│   ├── Cargo.toml         # Cargo configuration
│   └── src/
│       ├── lib.rs         # Main library file
│       ├── envelope.rs    # Envelope struct
│       ├── header.rs      # Header struct
│       ├── validator.rs   # Validator struct
│       └── schema_loader.rs # Schema loader struct
└── schemas/               # Schema files organized by domain and version
```

## Java Implementation

### Building

```bash
cd java
mvn clean package
```

The packaged JAR will include all schema files as resources, making deployment easier.

### Usage

#### Basic Usage

```java
import net.hydrius.pacts.*;

// Create a header with authentication
Header header = new Header("1", "player", "application/json", "your-auth-token");

// Create data
Map<String, Object> playerRequestData = new HashMap<>();
playerRequestData.put("target_id", "885c3cca-d537-4478-84f0-580deb1a6f05");
playerRequestData.put("request_type", "PLAYER_JOIN");
playerRequestData.put("date", header.getTimestamp());

// Create envelope
Envelope envelope = new Envelope(header, playerRequestData);

// Validate
Validator validator = new Validator();
ValidationResult result = validator.validate(envelope);

if (result.isValid()) {
    System.out.println("Validation successful");
    // Send to RabbitMQ or other messaging system
} else {
    System.out.println("Validation failed: " + result.getErrorMessage());
}
```

#### Directory-Based Schema Loading

You can also load schemas by directory structure, where the version is automatically extracted from `v{number}` directories.

```java
import net.hydrius.pacts.SchemaLoader;
import com.fasterxml.jackson.databind.JsonNode;

// Create a schema loader
SchemaLoader schemaLoader = new SchemaLoader("schemas");

// Load a schema using directory structure: domain/category/schemaName
// This automatically finds the version from v{number} directories
JsonNode inventorySchema = schemaLoader.loadSchemaByDirectory("bees", "inventory", "inventory_item");
JsonNode playerSchema = schemaLoader.loadSchemaByDirectory("bees", "player", "player_request");

// Schemas are cached for performance
JsonNode cachedSchema = schemaLoader.loadSchemaByDirectory("bees", "inventory", "inventory_item");
```

**Directory Structure Support:**

- Automatically finds version directories (e.g., `v1`, `v2`)
- Follows path: `schemas/{domain}/{version}/{category}/{schemaName}.json`
- Includes caching for improved performance
- Returns `null` for non-existent schemas
- Automatically loads from packaged resources if not found on filesystem

#### Using PactsService

The PactsService class provides convenient methods for common operations:

```java
import net.hydrius.pacts.*;

// Create service
PactsService pactsService = new PactsService();

// Create and validate envelope with authentication
Envelope envelope = pactsService.createEnvelope("1", "player_request", playerData, "auth-token");

ValidationResult result = pactsService.validate(envelope);
if (result.isValid()) {
    String json = pactsService.toJson(envelope);
    // Send json to RabbitMQ or other messaging system
}

// Validate data against a specific schema
ValidationResult dataResult = pactsService.validateData(
    inventoryData, "bees", "inventory", "inventory_item"
);
```

#### Spring Boot Integration

For Spring Boot applications, create a configuration bean:

```java
import org.springframework.context.annotation.*;
import net.hydrius.pacts.*;

@Configuration
public class PactsConfiguration {
    
    @Value("${pacts.schema.base-path:schemas}")
    private String schemaBasePath;
    
    @Bean
    public SchemaLoader schemaLoader() {
        return new SchemaLoader(schemaBasePath);
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
```

Then use it in your services:

```java
@Service
public class GameService {
    @Autowired
    private PactsService pactsService;
    
    @Autowired
    private RabbitTemplate rabbitTemplate;
    
    public void sendPlayerJoinRequest(String playerId, String authToken) throws Exception {
        Map<String, Object> data = new HashMap<>();
        data.put("target_id", playerId);
        data.put("request_type", "PLAYER_JOIN");
        data.put("date", Instant.now().toString());
        
        // Create and validate envelope
        Envelope envelope = pactsService.createEnvelope("1", "player_request", data, authToken);
        
        ValidationResult result = pactsService.validate(envelope);
        if (result.isValid()) {
            // Convert to JSON and create AMQP message
            String json = pactsService.toJson(envelope);
            
            MessageProperties props = new MessageProperties();
            props.setContentType("application/json");
            props.setHeader("schema-id", envelope.getHeader().getSchemaId());
            props.setHeader("schema-version", envelope.getHeader().getSchemaVersion());
            props.setHeader("auth-token", envelope.getHeader().getAuthToken());
            
            Message message = MessageBuilder
                .withBody(json.getBytes())
                .andProperties(props)
                .build();
                
            rabbitTemplate.send("game.exchange", "player.join", message);
        } else {
            throw new IllegalArgumentException("Invalid player join request: " + result.getErrorMessage());
        }
    }
    
    @RabbitListener(queues = "player.responses")
    public void handlePlayerResponse(Message message) throws Exception {
        String json = new String(message.getBody());
        Envelope envelope = pactsService.parseEnvelope(json);
        
        // Validate the received message
        ValidationResult result = pactsService.validate(envelope);
        if (result.isValid()) {
            // Check authentication if needed
            String authToken = envelope.getHeader().getAuthToken();
            if (authToken != null && !isValidToken(authToken)) {
                log.error("Invalid auth token in response");
                return;
            }
            
            // Process the valid response
            processPlayerResponse(envelope);
        } else {
            log.error("Received invalid response: {}", result.getErrorMessage());
        }
    }
}
```

Configuration properties:

```properties
# application.properties
pacts.schema.base-path=schemas
pacts.schema.cache.enabled=true
```

## Rust Implementation

### Building

```bash
cd rust
cargo build
```

### Usage

#### Basic Usage

```rust
use pacts::{Envelope, Header, Validator, ValidationResult};
use serde_json::json;

// Create a header with authentication
let header = Header::with_auth(
    "1".to_string(), 
    "player".to_string(),
    Some("application/json".to_string()),
    "your-auth-token".to_string()
);

// Create data
let player_request_data = json!({
    "target_id": "885c3cca-d537-4478-84f0-580deb1a6f05",
    "request_type": "PLAYER_JOIN",
    "date": header.timestamp().to_string()
});

// Create envelope
let envelope = Envelope::new(header, player_request_data);

// Validate
let validator = Validator::new();
let result = validator.validate(&envelope);

if result.is_valid() {
    println!("Validation successful");
    // Send to RabbitMQ using lapin or other AMQP client
} else {
    println!("Validation failed: {}", result.error_message());
}
```

#### Directory-Based Schema Loading

You can also load schemas by directory structure, where the version is automatically extracted from `v{number}` directories.

```rust
use pacts::schema_loader::SchemaLoader;
use serde_json::Value;

// Create a schema loader
let mut schema_loader = SchemaLoader::with_base_path("schemas".to_string());

// Load a schema using directory structure: domain/category/schemaName
// This automatically finds the version from v{number} directories
let inventory_schema = schema_loader.load_schema_by_directory("bees", "inventory", "inventory_item");
let player_schema = schema_loader.load_schema_by_directory("bees", "player", "player_request");

// Schemas are cached for performance
let cached_schema = schema_loader.load_schema_by_directory("bees", "inventory", "inventory_item");
```

**Directory Structure Support:**

- Automatically finds version directories (e.g., `v1`, `v2`)
- Follows path: `schemas/{domain}/{version}/{category}/{schemaName}.json`
- Includes caching for improved performance
- Returns `Option<Value>` for proper error handling

#### Using PactsService

The PactsService struct provides convenient methods:

```rust
use pacts::{PactsService, ValidationResult};
use serde_json::json;

// Create service
let service = PactsService::new();

// Create and validate envelope with authentication
let data = json!({
    "target_id": "player-123",
    "request_type": "PLAYER_JOIN",
    "date": chrono::Utc::now().to_string()
});

let envelope = service.create_envelope_with_auth(
    "1".to_string(),
    "player_request".to_string(),
    data,
    "auth-token".to_string()
);

let result = service.validate(&envelope);
if result.is_valid() {
    let json = serde_json::to_string(&envelope).unwrap();
    // Send json to RabbitMQ or other messaging system
}

// Validate data against a specific schema
let inventory_data = json!({
    "slot": 1,
    "material": "diamond_sword",
    "amount": 1
});

let data_result = service.validate_data(
    &inventory_data,
    "bees",
    "inventory", 
    "inventory_item"
);
```

#### Actix-Web Integration Example

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use pacts::{Envelope, Header, Validator, SchemaLoader};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct PlayerRequest {
    target_id: String,
    request_type: String,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

struct AppState {
    validator: Arc<Validator>,
    auth_token: String,
}

async fn handle_player_request(
    data: web::Json<PlayerRequest>,
    state: web::Data<AppState>,
    auth_header: web::Header<String>,
) -> HttpResponse {
    // Verify authentication
    if auth_header.as_str() != &state.auth_token {
        return HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Invalid authentication token".to_string(),
        });
    }
    
    // Create envelope with authentication
    let header = Header::with_auth(
        "1".to_string(),
        "player_request".to_string(),
        Some("application/json".to_string()),
        auth_header.to_string(),
    );
    
    let envelope = Envelope::new(header, serde_json::to_value(&data.into_inner()).unwrap());
    
    // Validate
    let result = state.validator.validate(&envelope);
    
    if result.is_valid() {
        // Process the request (send to RabbitMQ, etc.)
        HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Player request processed successfully".to_string(),
        })
    } else {
        HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Validation failed: {}", result.error_message()),
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema_loader = SchemaLoader::with_base_path("schemas".to_string());
    let validator = Arc::new(Validator::with_schema_loader(schema_loader));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                validator: validator.clone(),
                auth_token: "your-server-auth-token".to_string(),
            }))
            .route("/player/request", web::post().to(handle_player_request))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Schema Format

Schemas are JSON files that define the structure and validation rules for data. The system supports basic JSON Schema features:

- `type`: Specifies the expected data type (object, array, string, number, boolean, null)
- `required`: Array of required field names
- `properties`: Object defining field types (basic support)

Example schema (`schemas/bees/v1/inventory/inventory_item.json`):

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "name": { "type": "string" },
    "quantity": { "type": "number" },
    "active": { "type": "boolean" }
  },
  "required": ["id", "name", "quantity"]
}
```

## Schema Organization

Schemas are organized in a hierarchical directory structure:

```text
schemas/
├── {domain}/              # Domain (e.g., bees, moderation, network)
│   └── v{number}/         # Version directory (e.g., v1, v2)
│       ├── {category}/    # Category (e.g., inventory, player, profile)
│       │   ├── {schema_name}.json
│       │   └── {schema_name}.json
│       └── {category}/
│           └── {schema_name}.json
└── {domain}/
    └── v{number}/
        └── {category}/
            └── {schema_name}.json
```

This structure allows for:

- **Domain separation**: Different domains (bees, moderation, etc.)
- **Version management**: Automatic version detection from `v{number}` directories
- **Category organization**: Logical grouping within domains
- **Schema naming**: Clear, descriptive schema names

## Features

- **Authentication Support**: Built-in auth token handling in Headers
- **Envelope Pattern**: Wraps data with metadata for validation and routing
- **Header**: Contains schema version, ID, timestamp, content type, and auth token
- **Validator**: Validates envelopes and data against schemas
- **SchemaLoader**: Loads schemas from files or packaged resources with caching
- **Directory-based loading**: Automatic schema discovery by domain/category/name
- **Version detection**: Automatic version extraction from directory structure
- **Spring Boot Integration**: Ready-to-use configuration and service classes
- **Resource Packaging**: Schemas are packaged with JAR for easy deployment
- **Basic validation**: Type checking and required field validation
- **Cross-platform**: Java and Rust implementations
- **Caching**: Performance optimization for frequently accessed schemas
- **RabbitMQ Ready**: Examples for AMQP message conversion and handling

## Testing

### Rust Tests

```bash
cd rust
cargo test
```

### Java Tests

```bash
cd java
mvn test
```

The test suite includes comprehensive coverage for:

- Directory-based schema loading
- Non-existent schema handling
- Caching functionality
- Error cases and edge conditions
