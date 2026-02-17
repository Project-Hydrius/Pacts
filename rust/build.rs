use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let source_file = Path::new(&manifest_dir).join("resources/sources.yaml");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_file = Path::new(&out_dir).join("sources.yaml");

    if source_file.exists() {
        fs::copy(&source_file, &dest_file).expect("Failed to copy sources.yaml");
        println!("cargo:rerun-if-changed={}", source_file.display());
    } else {
        panic!(
            "sources.yaml not found in resources folder: {}",
            source_file.display()
        );
    }
}
