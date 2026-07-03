use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    let header = r"C:\Users\lross\development\e384c\include\e384c.h";
    // this needs to point to the .lib (even if the library is a dll, it exposes names of fucntions)
    let lib_path =
        r"C:\Users\lross\development\e384c\build\Desktop_Qt_6_7_3_MSVC2022_64bit-Release";
    println!("cargo:rustc-link-search={lib_path}");

    // Tell cargo to tell rustc to e384c
    // shared library.
    println!("cargo:rustc-link-lib=e384c");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(header)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
