use std::env;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("shaders");

    println!("cargo:rerun-if-changed={}", path.display());
}
