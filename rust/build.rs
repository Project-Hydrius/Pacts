use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the project root directory (parent of the directory containing Cargo.toml)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let project_root = Path::new(&manifest_dir).parent().expect("Failed to get project root");
    
    // Source file path (in rust/resources folder)
    let source_file = Path::new(&manifest_dir).join("resources/sources.yaml");
    
    // Output directory (target directory)
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let target_dir = Path::new(&out_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to determine target directory");
    
    // Destination path
    let dest_file = target_dir.join("sources.yaml");
    
    // Copy the file if it exists
    if source_file.exists() {
        fs::copy(&source_file, &dest_file).expect("Failed to copy sources.yaml");
        println!("cargo:rerun-if-changed={}", source_file.display());
    } else {
        panic!("sources.yaml not found in resources folder: {}", source_file.display());
    }
}
