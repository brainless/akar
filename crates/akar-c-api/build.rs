fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::Path::new(&crate_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    cbindgen::generate(&crate_dir)
        .expect("cbindgen failed to generate header")
        .write_to_file(workspace_root.join("akar.h"));
}
