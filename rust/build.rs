use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the directory where the build script is located
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let schemas_dir = Path::new(&manifest_dir).parent().unwrap().join("schemas");
    let version_file = schemas_dir.join("VERSION");

    // Tell Cargo to rerun this build script if the schemas directory changes
    println!("cargo:rerun-if-changed={}", schemas_dir.display());

    // Pass schema path into build
    if schemas_dir.exists() {
        println!("cargo:rustc-env=SCHEMAS_DIR={}", schemas_dir.display());
    }

    // Read schema version and expose it
    if version_file.exists() {
        let version = fs::read_to_string(version_file)
            .expect("Failed to read schema version file")
            .trim()
            .to_string();
        println!("cargo:rustc-env=SCHEMA_VERSION={}", version);
    }
}
