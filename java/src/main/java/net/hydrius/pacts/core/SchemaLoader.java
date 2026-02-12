package net.hydrius.pacts.core;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;
import java.util.logging.Logger;
import java.util.zip.ZipEntry;
import java.util.zip.ZipException;
import java.util.zip.ZipInputStream;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;

/**
 * SchemaLoader class that loads schemas.
 * 
 * This class supports loading schemas from local and remote resources.
 */
public class SchemaLoader {

    private final ObjectMapper objectMapper;
    private final Map<String, JsonNode> cache;
    private static final Logger logger = Logger.getLogger(SchemaLoader.class.getName());

    private final String schemaRoot;
    private final String domain;
    private final String version;

    private static final int CONNECTION_TIMEOUT_MS = 15_000;
    private static final int READ_TIMEOUT_MS = 30_000;
    private static final long MAX_ENTRY_SIZE = 10 * 1024 * 1024; // 10 MB per ZIP entry

    /**
     * Creates a new SchemaLoader.
     *
     * @param schemaRoot the directory containing the schemas
     * @param domain     the domain of the schema
     * @param version    the version of the schema
     * @throws IllegalArgumentException if the schema root, domain, or version
     *                                  is null
     */
    public SchemaLoader(String schemaRoot, String domain, String version) {
        if (schemaRoot == null || domain == null || version == null) {
            throw new IllegalArgumentException("Schema root, domain, and version must be specified.");
        }

        this.objectMapper = new ObjectMapper();
        this.objectMapper.registerModule(new JavaTimeModule());
        this.cache = new HashMap<>();
        this.schemaRoot = schemaRoot;
        this.domain = domain;
        this.version = version;

        try {
            loadRemoteSchemas();
        } catch (IOException e) {
            logger.warning(() -> "Remote schema loading failed, falling back to local file system only: " + e.getMessage());
        }
    }

    /**
     * Loads a schema from cache or local resources.
     *
     * @param category The schema category (e.g., "player")
     * @param name     The schema name (e.g., "player_request")
     * @return The parsed JSON schema node or null if not found.
     */
    public JsonNode loadSchema(String category, String name) {
        String cacheKey = domain + "/" + version + "/" + category + "/" + name;

        if (cache.containsKey(cacheKey)) {
            return cache.get(cacheKey);
        }

        // Try to load from file system
        try {
            Path schemaPath = Paths.get(schemaRoot, domain, version, category, name + ".json");
            if (Files.exists(schemaPath)) {
                JsonNode schema = objectMapper.readTree(schemaPath.toFile());
                cache.put(cacheKey, schema);
                return schema;
            }
        } catch (IOException e) {
            logger.warning(() -> "Failed to load schema from file system: " + e.getMessage());
        }

        return null;
    }

    /**
     * Loads schemas from remote ZIP files specified in application.yml.
     * 
     * This method processes ZIP files in memory without writing to disk,
     * extracting all JSON schema files and storing them in an in-memory cache.
     * 
     * The loader supports multiple source URLs for redundancy, trying each one
     * in order until a successful load occurs.
     * 
     * @throws IOException if sources could not be read or found
     */
    private void loadRemoteSchemas() throws IOException {
        JsonNode settings = null;
        try (InputStream stream = getClass().getClassLoader().getResourceAsStream("application.yml")) {
            if (stream == null) {
                logger.warning("Application Settings not found");
            } else {
                ObjectMapper yamlMapper = new ObjectMapper(new YAMLFactory());
                JsonNode parsed = yamlMapper.readTree(stream);

                if (parsed == null || !parsed.has("sources")) {
                    logger.warning("application.yml is missing 'sources' key");
                } else {
                    JsonNode sourcesNode = parsed.get("sources");
                    if (!sourcesNode.isArray()) {
                        logger.warning("'sources' key in application.yml is not an array");
                    } else {
                        settings = objectMapper.createObjectNode();
                        ((ObjectNode) settings).set("sources", objectMapper.convertValue(sourcesNode, JsonNode.class));
                    }
                }
            }
        } catch (Exception e) {
            logger.warning(() -> "Failed to parse application settings: " + e.getMessage());
        }

        boolean sourcesLoaded = false;
        if (settings != null) {
            JsonNode sourcesNode = settings.get("sources");
            for (JsonNode node : sourcesNode) {
                String source = node.asText();
                HttpURLConnection connection = null;
                try {
                    logger.info(() -> "Attempting to load schemas from: " + source);
                    URL url = new URL(source);
                    connection = (HttpURLConnection) url.openConnection();
                    connection.setRequestMethod("GET");
                    connection.setConnectTimeout(CONNECTION_TIMEOUT_MS);
                    connection.setReadTimeout(READ_TIMEOUT_MS);

                    try (InputStream in = connection.getInputStream();
                         ZipInputStream zipIn = new ZipInputStream(in)) {

                        ZipEntry entry = zipIn.getNextEntry();
                        while (entry != null) {
                            if (!entry.isDirectory() && entry.getName().endsWith(".json")) {
                                ByteArrayOutputStream baos = new ByteArrayOutputStream();
                                byte[] buffer = new byte[1024];
                                long totalRead = 0;
                                int len;
                                while ((len = zipIn.read(buffer)) > 0) {
                                    totalRead += len;
                                    if (totalRead > MAX_ENTRY_SIZE) {
                                        throw new IOException("ZIP entry exceeds maximum allowed size: " + entry.getName());
                                    }
                                    baos.write(buffer, 0, len);
                                }

                                byte[] content = baos.toByteArray();
                                JsonNode schema = objectMapper.readTree(content);

                                String entryPath = entry.getName();
                                int lastSlash = entryPath.lastIndexOf('/');
                                String fileName = lastSlash >= 0 ? entryPath.substring(lastSlash + 1) : entryPath;
                                String categoryName = lastSlash >= 0 ? entryPath.substring(0, lastSlash) : "";

                                String[] pathParts = categoryName.split("/");
                                if (pathParts.length >= 3) {
                                    String entryDomain = pathParts[pathParts.length - 3];
                                    String entryVersion = pathParts[pathParts.length - 2];
                                    String entryCategory = pathParts[pathParts.length - 1];
                                    String schemaName = fileName.substring(0, fileName.length() - 5);

                                    String cacheKey = entryDomain + "/" + entryVersion + "/" + entryCategory + "/"
                                            + schemaName;
                                    cache.put(cacheKey, schema);
                                    logger.info(() -> "Loaded schema into cache: " + cacheKey);
                                }
                            }

                            zipIn.closeEntry();
                            entry = zipIn.getNextEntry();
                        }
                    }

                    sourcesLoaded = true;
                    logger.info(() -> "Successfully loaded schemas from: " + source);
                    break;
                } catch (ZipException e) {
                    logger.warning(() -> "Failed to process ZIP from source: " + source + ", trying next source if available");
                } catch (IOException e) {
                    logger.warning(() -> "IO Exception while processing source: " + source + ", trying next source if available");
                } finally {
                    if (connection != null) {
                        connection.disconnect();
                    }
                }
            }
        }

        if (!sourcesLoaded) {
            throw new IOException("Sources could not be read or found to populate schemas.");
        }
    }

    /**
     * Clears all cached schemas.
     */
    public void clearCache() {
        cache.clear();
    }

    /**
     * Gets the schema root.
     *
     * @return the schema root
     */
    public String getSchemaRoot() {
        return schemaRoot;
    }

    /**
     * Gets the domain.
     *
     * @return the domain
     */
    public String getDomain() {
        return domain;
    }

    /**
     * Gets the version.
     *
     * @return the version
     */
    public String getVersion() {
        return version;
    }

    /**
     * Gets the parsed version.
     *
     * @return the parsed version
     */
    public int getParsedVersion() {
        return Integer.parseInt(version.replace("v", ""));
    }
}
