fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::Path::new(&crate_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let header = workspace_root.join("akar.h");
    cbindgen::generate(&crate_dir)
        .expect("cbindgen failed to generate header")
        .write_to_file(&header);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    cc::Build::new()
        .file(workspace_root.join("crates/akar-c-api/tests/text_edit.c"))
        .include(workspace_root)
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .cargo_metadata(false)
        .compile("akar_c_api_text_edit_tests");
    println!("cargo:rustc-link-search=native={out_dir}");
}
