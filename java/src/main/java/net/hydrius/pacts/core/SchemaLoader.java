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
import java.util.Scanner;
import java.util.logging.Logger;
import java.util.zip.ZipEntry;
import java.util.zip.ZipException;
import java.util.zip.ZipInputStream;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;

/**
 * SchemaLoader class that loads schemas that are bundled with Pacts.
 * 
 * This class supports loading schemas from the file system or embedded resources.
 * Remote ZIP loading functionality is available through a separate method.
 */
public class SchemaLoader {

    private final ObjectMapper objectMapper;
    private final Map<String, JsonNode> cache;
    private static final Logger logger = Logger.getLogger(SchemaLoader.class.getName());

    private final String schemaRoot;
    private final String domain;
    private final String version;

    /**
     * Creates a new SchemaLoader.
     *
     * @param schemaRoot the directory containing the schemas
     * @param domain     the domain of the schema
     * @param version    the version of the schema
     * @throws IllegalArgumentException if the schema root, domain, or version
     *                                  is null
     * @throws IOException if the schemas could not be loaded
     */
    public SchemaLoader(String schemaRoot, String domain, String version) throws IllegalArgumentException, IOException {
        if (schemaRoot == null || domain == null || version == null) {
            throw new IllegalArgumentException("Schema root, domain, and version must be specified.");
        }

        this.objectMapper = new ObjectMapper();
        this.objectMapper.registerModule(new JavaTimeModule());
        this.cache = new HashMap<>();
        this.schemaRoot = schemaRoot;
        this.domain = domain;
        this.version = version;

        loadRemoteSchemas();
    }

    /**
     * Loads a schema from cache, file system, or embedded resources.
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
            logger.warning("Failed to load schema from file system: " + e.getMessage());
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
    public void loadRemoteSchemas() throws IOException {
        JsonNode settings = null;
        try (InputStream stream = getClass().getClassLoader().getResourceAsStream("application.yml")) {
            if (stream == null) {
                logger.warning("Application Settings not found");
            } else {
                // Load YAML settings using SnakeYAML or similar
                // For now, we'll manually parse the YAML structure
                Scanner scanner = new Scanner(stream).useDelimiter("\\A");
                String yamlContent = scanner.hasNext() ? scanner.next() : "";

                // Simple parsing for sources array - in a real implementation you would use a
                // proper YAML parser
                if (yamlContent.contains("sources:")) {
                    settings = objectMapper.createObjectNode();
                    ArrayNode sourcesArray = objectMapper.createArrayNode();

                    // Extract sources from YAML (simplified parsing)
                    String[] lines = yamlContent.split("\n");
                    for (String line : lines) {
                        if (line.trim().startsWith("- ")) {
                            String source = line.trim().substring(2).trim();
                            if (source.startsWith("\"") && source.endsWith("\"")) {
                                source = source.substring(1, source.length() - 1);
                            }
                            sourcesArray.add(source);
                        }
                    }

                    ((ObjectNode) settings).set("sources", sourcesArray);
                }
            }
        } catch (Exception e) {
            logger.warning("Failed to parse application settings: " + e.getMessage());
        }

        boolean sourcesLoaded = false;
        if (settings != null) {
            ArrayNode sourcesNode = (ArrayNode) settings.get("sources");
            for (JsonNode node : sourcesNode) {
                String source = node.asText();
                try {
                    logger.info("Attempting to load schemas from: " + source);
                    URL url = new URL(source);
                    HttpURLConnection connection = (HttpURLConnection) url.openConnection();
                    connection.setRequestMethod("GET");
                    InputStream in = connection.getInputStream();
                    ZipInputStream zipIn = new ZipInputStream(in);
                    ZipEntry entry = zipIn.getNextEntry();

                    while (entry != null) {
                        // Process schema files from ZIP
                        if (!entry.isDirectory() && entry.getName().endsWith(".json")) {
                            // Extract schema content
                            ByteArrayOutputStream baos = new ByteArrayOutputStream();
                            byte[] buffer = new byte[1024];
                            int len;
                            while ((len = zipIn.read(buffer)) > 0) {
                                baos.write(buffer, 0, len);
                            }

                            // Parse and store schema in cache
                            byte[] content = baos.toByteArray();
                            JsonNode schema = objectMapper.readTree(content);

                            // Extract category and name from path
                            String entryPath = entry.getName();
                            // Remove leading path parts to get relative path
                            int lastSlash = entryPath.lastIndexOf('/');
                            String fileName = lastSlash >= 0 ? entryPath.substring(lastSlash + 1) : entryPath;
                            String categoryName = lastSlash >= 0 ? entryPath.substring(0, lastSlash) : "";

                            // Further split category if needed
                            String[] pathParts = categoryName.split("/");
                            if (pathParts.length >= 3) {
                                String entryDomain = pathParts[pathParts.length - 3];
                                String entryVersion = pathParts[pathParts.length - 2];
                                String entryCategory = pathParts[pathParts.length - 1];

                                // Remove .json extension from filename
                                String schemaName = fileName.substring(0, fileName.length() - 5);

                                // Store in cache with proper key format
                                String cacheKey = entryDomain + "/" + entryVersion + "/" + entryCategory + "/"
                                        + schemaName;
                                cache.put(cacheKey, schema);
                                logger.info("Loaded schema into cache: " + cacheKey);
                            }
                        }

                        // Close current entry and move to next
                        zipIn.closeEntry();
                        entry = zipIn.getNextEntry();
                    }

                    sourcesLoaded = true;
                    zipIn.close();
                    in.close();
                    logger.info("Successfully loaded schemas from: " + source);
                    break;
                } catch (ZipException e) {
                    logger.warning(
                            "Failed to process ZIP from source: " + source + ", trying next source if available");
                } catch (IOException e) {
                    logger.warning(
                            "IO Exception while processing source: " + source + ", trying next source if available");
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
