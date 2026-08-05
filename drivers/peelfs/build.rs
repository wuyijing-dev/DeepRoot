use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let linker = manifest.join("linker.ld");
    println!("cargo:rerun-if-changed={}", linker.display());
    println!("cargo:rustc-link-arg=-T{}", linker.display());
}
