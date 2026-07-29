use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let src = PathBuf::from(&manifest).join("cloak/libzcloak.c");
    let so = PathBuf::from(&out_dir).join("libzcloak.so");

    println!("cargo:rerun-if-changed={}", src.display());

    let cc_path = cc::Build::new().get_compiler().path().to_path_buf();
    let status = Command::new(&cc_path)
        .args(["-shared", "-fPIC", "-O2"])
        .arg("-o")
        .arg(&so)
        .arg(&src)
        .arg("-ldl")
        .status()
        .expect("failed to invoke C compiler for libzcloak");
    assert!(status.success(), "compiling libzcloak.so failed");

    println!("cargo:rustc-env=ZCLOAK_LIB={}", so.display());

    tauri_build::build();
}
