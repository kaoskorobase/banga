extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!(
        "cargo:rustc-link-search=native={}",
        "../samplecount/methcla/build/src"
    );
    println!("cargo:rustc-link-lib=methcla");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/methcla_wrapper.h")
        .clang_arg("-I../samplecount/methcla/include")
        .whitelist_type("Methcla_.*")
        .whitelist_function("methcla_.*")
        .prepend_enum_name(false)
        .bitfield_enum("Methcla_BusMappingFlags")
        .bitfield_enum("Methcla_NodeDoneFlags")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate methcla bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("methcla_bindings.rs"))
        .expect("Couldn't write methcla bindings!");
}
