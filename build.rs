use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mir_src = PathBuf::from(manifest_dir).join("mir");

    // --- 1. Compile C Code ---
    let mut build = cc::Build::new();
    
    build
        .include(&mir_src)
        // MIR core files
        .file(mir_src.join("mir.c"))
        .file(mir_src.join("mir-gen.c")) 
        // Standard flags for MIR
        .flag("-std=gnu11") 
        .flag("-fsigned-char")
        .flag("-fno-tree-sra")
        .flag("-fno-ipa-cp-clone")
        .flag("-Wno-abi");

    // Optimization
    if env::var("PROFILE").unwrap() == "release" {
        build.flag("-O3");
    }

    build.compile("mir");

    // --- 2. Generate Rust Bindings ---
    println!("cargo:rerun-if-changed=mir/mir.h");
    
    let bindings = bindgen::Builder::default()
        .header("mir/mir.h")
        .header("mir/mir-gen.h")
        // Important: Tell bindgen where to look for includes
        .clang_arg(format!("-I{}", mir_src.display()))
        // Use the same standard as compilation
        .clang_arg("-std=gnu11") 
        // Whitelist what we need to keep compile times low
        .allowlist_function("MIR_.*")
        .allowlist_type("MIR_.*")
        .allowlist_var("MIR_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
        
    // Link against math library
    println!("cargo:rustc-link-lib=m");
}
