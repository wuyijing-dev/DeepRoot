use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let linker = manifest.join("linker.ld");
    let frames = manifest.join("frames.ba01");
    let mp4 = manifest.join("badapple.mp4");
    let gen = manifest.join("gen_frames.py");

    println!("cargo:rerun-if-changed={}", linker.display());
    println!("cargo:rerun-if-changed={}", frames.display());
    println!("cargo:rerun-if-changed={}", gen.display());
    println!("cargo:rerun-if-changed={}", mp4.display());
    println!("cargo:rustc-link-arg=-T{}", linker.display());

    if !frames.exists() {
        if !mp4.exists() {
            panic!(
                "missing {} — place badapple.mp4 here or commit frames.ba01",
                frames.display()
            );
        }
        let st = Command::new("python3")
            .current_dir(&manifest)
            .arg(&gen)
            .status()
            .expect("run gen_frames.py");
        assert!(st.success(), "gen_frames.py failed");
        assert!(frames.exists(), "frames.ba01 not produced");
    }
}
