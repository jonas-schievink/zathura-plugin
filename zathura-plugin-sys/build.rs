use {
    bindgen, pkg_config,
    std::{env, iter, path::PathBuf},
};

fn main() {
    let cairo = pkg_config::Config::new().probe("cairo").unwrap();
    let include_paths = std::env::join_paths(cairo.include_paths).unwrap();
    let include_paths = include_paths.to_string_lossy();

    let include_paths = include_paths
        .split(':')
        .chain(iter::once("headers"))
        .map(|s| format!("-I{}", s));

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
