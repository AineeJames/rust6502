use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Define the subdirectory path relative to the project root
    let subdirectory = "examples/";

    // Get the root directory of the repository
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Construct the path to the subdirectory
    let subdir_path = Path::new(&root_dir).join(subdirectory);

    // Check if the subdirectory exists
    if !subdir_path.exists() {
        panic!("Subdirectory {} does not exist", subdirectory);
    }

    // Run `make` in the subdirectory
    let status = Command::new("make")
        .current_dir(subdir_path.clone())
        .status()
        .expect("Failed to run `make` command");

    if !status.success() {
        panic!("`make` command failed with status: {}", status);
    }
    println!("cargo:rerun-if-changed=examples/*");
}
