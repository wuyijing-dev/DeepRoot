use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest.parent().unwrap().to_path_buf();
    let linker = manifest.join("linker.ld");
    println!("cargo:rerun-if-changed={}", linker.display());
    println!("cargo:rerun-if-changed=../VERSION");
    println!("cargo:rustc-link-arg=-T{}", linker.display());

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let user_target = workspace.join("target/user-build");
    let triple = "riscv64gc-unknown-none-elf";

    for (pkg, bin, src_dir) in [
        ("deeproot-init", "deeproot-init", "user/init"),
        ("deeproot-console", "deeproot-console", "user/console"),
        ("deeproot-ping", "deeproot-ping", "user/ping"),
        ("deeproot-hello", "deeproot-hello", "user/hello"),
        ("deeproot-shell", "deeproot-shell", "user/shell"),
        ("deeproot-badapple", "deeproot-badapple", "user/badapple"),
        ("deeproot-moddemo", "deeproot-moddemo", "user/moddemo"),
    ] {
        println!("cargo:rerun-if-changed=../{}/src/main.rs", src_dir);
        println!("cargo:rerun-if-changed=../{}/linker.ld", src_dir);
        println!("cargo:rerun-if-changed=../{}/build.rs", src_dir);
        if src_dir == "user/badapple" {
            println!("cargo:rerun-if-changed=../user/badapple/frames.ba01");
        }

        let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .current_dir(&workspace)
            .env("CARGO_TARGET_DIR", &user_target)
            .args(["build", "-p", pkg, "--release", "--target", triple])
            .status()
            .unwrap_or_else(|e| panic!("spawn cargo {}: {}", pkg, e));
        assert!(status.success(), "failed building {}", pkg);

        let elf = user_target.join(triple).join("release").join(bin);
        assert!(elf.exists(), "missing {}", elf.display());
        std::fs::copy(&elf, out.join(bin)).expect("copy user elf");
    }
}
