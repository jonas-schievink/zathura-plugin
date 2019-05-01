use std::env;
use std::path::PathBuf;
use bindgen;
use pkg_config;

fn main() {
    let zathura = pkg_config::Config::new().probe("zathura").unwrap();
    let include_paths = std::env::join_paths(zathura.include_paths).unwrap();
    let include_paths = include_paths.to_string_lossy();
    println!("cargo:include={}", include_paths);

    let include_paths = include_paths.split(':').map(|s| format!("-I{}", s));

    let bindings = bindgen::Builder::default()
        .clang_args(include_paths)
        .whitelist_type("zathura_.*")
        .whitelist_function("zathura_.*")
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
