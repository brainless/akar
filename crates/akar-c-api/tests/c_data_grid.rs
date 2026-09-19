#[link(name = "akar_c_api_data_grid_tests", kind = "static")]
unsafe extern "C" {
    fn akar_run_data_grid_c_tests();
}

#[test]
fn c_data_grid_api() {
    std::hint::black_box(akar_c_api::akar_ctx_mock as unsafe extern "C" fn() -> _);
    unsafe { akar_run_data_grid_c_tests() };
}
