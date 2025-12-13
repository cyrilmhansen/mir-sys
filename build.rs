use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let vendor = PathBuf::from(manifest_dir).join("mir");

    // 1. Compile C Code
    let mut build = cc::Build::new();

    build
        .include(&vendor)
        .file(vendor.join("mir.c"))
        .file(vendor.join("mir-gen.c")) // This internally includes mir-gen-x86_64.c etc.
        .flag("-std=gnu11") // Required for MIR GNU extensions
        .flag("-fsigned-char") // Critical for ARM/Android
        .flag("-fno-tree-sra")
        .flag("-fno-ipa-cp-clone")
        .flag("-fPIC")
        // Suppress C compiler warnings
        .flag("-Wno-abi")
        .flag("-Wno-missing-field-initializers")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-variable")
        .flag("-Wno-sign-compare")
        .flag("-Wno-implicit-function-declaration")
        .warnings(false);

    if env::var("PROFILE").unwrap() == "release" {
        build.flag("-O3");
    }

    build.compile("mir");

    // 2. Generate Rust Bindings
    println!("cargo:rerun-if-changed=mir/mir.h");

    let bindings = bindgen::Builder::default()
        // Use a wrapper header to handle include deduplication automatically
        .header_contents("wrapper.h", "#include \"mir.h\"\n#include \"mir-gen.h\"")
        .clang_arg(format!("-I{}", vendor.display()))
        .clang_arg("-std=gnu11")
        .allowlist_function("MIR_.*")
        .allowlist_function("_MIR_.*") // Required for initialization
        .allowlist_type("MIR_.*")
        .allowlist_var("MIR_.*")
        .allowlist_type("DLIST_.*")
        .allowlist_type("VARR_.*")
        // Let bindgen handle FILE and va_list (do not blocklist them)
        .layout_tests(false)
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-lib=m");
}
