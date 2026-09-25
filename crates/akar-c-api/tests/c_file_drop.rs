#[link(name = "akar_c_api_file_drop_tests", kind = "static")]
unsafe extern "C" {
    fn akar_run_file_drop_c_tests();
}

#[test]
fn c_file_drop_api() {
    std::hint::black_box(akar_c_api::akar_ctx_mock as unsafe extern "C" fn() -> _);
    unsafe { akar_run_file_drop_c_tests() };
}
