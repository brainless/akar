#[link(name = "akar_c_api_text_edit_tests", kind = "static")]
unsafe extern "C" {
    fn akar_run_text_edit_c_tests();
}

#[test]
fn c_text_edit_api() {
    std::hint::black_box(akar_c_api::akar_ctx_mock as unsafe extern "C" fn() -> _);
    unsafe { akar_run_text_edit_c_tests() };
}
