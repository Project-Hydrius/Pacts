use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the directory where the build script is located
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let schemas_dir = Path::new(&manifest_dir).parent().unwrap().join("schemas");

    // Tell Cargo to rerun this build script if the schemas directory changes
    println!("cargo:rerun-if-changed={}", schemas_dir.display());

    // Verify schemas directory exists
    if schemas_dir.exists() {
        println!("cargo:rustc-env=SCHEMAS_DIR={}", schemas_dir.display());
    }
}
