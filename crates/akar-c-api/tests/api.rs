use akar_c_api::{
    akar_add_child, akar_begin_frame, akar_ctx_free, akar_ctx_mock, akar_input_begin,
    akar_input_end, akar_layout_compute, akar_layout_rect, akar_new_fixed_leaf, akar_new_flex_col,
    akar_new_flex_row, akar_new_leaf, akar_push_mouse_button, akar_push_scroll, akar_set_mouse_pos,
};

#[test]
fn lifecycle_mock_create_free() {
    let ctx = unsafe { akar_ctx_mock() };
    assert!(!ctx.is_null());
    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn layout_flex_grow_leaf_fills_parent() {
    let ctx = unsafe { akar_ctx_mock() };

    let root = unsafe { akar_new_flex_col(ctx) };
    let child = unsafe { akar_new_leaf(ctx, 1.0) };
    unsafe { akar_add_child(ctx, root, child) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let rect = unsafe { akar_layout_rect(ctx, child) };
    assert!((rect.w - 800.0).abs() < 1.0, "child.w = {}", rect.w);
    assert!((rect.h - 600.0).abs() < 1.0, "child.h = {}", rect.h);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn layout_fixed_leaf_respected() {
    let ctx = unsafe { akar_ctx_mock() };

    let root = unsafe { akar_new_flex_row(ctx) };
    let child = unsafe { akar_new_fixed_leaf(ctx, 120.0, 40.0) };
    unsafe { akar_add_child(ctx, root, child) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let rect = unsafe { akar_layout_rect(ctx, child) };
    assert!((rect.w - 120.0).abs() < 1.0, "child.w = {}", rect.w);
    assert!((rect.h - 40.0).abs() < 1.0, "child.h = {}", rect.h);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn layout_two_flex_siblings_fill_row() {
    let ctx = unsafe { akar_ctx_mock() };

    let root = unsafe { akar_new_flex_row(ctx) };
    let left = unsafe { akar_new_leaf(ctx, 1.0) };
    let right = unsafe { akar_new_leaf(ctx, 1.0) };
    unsafe { akar_add_child(ctx, root, left) };
    unsafe { akar_add_child(ctx, root, right) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let lr = unsafe { akar_layout_rect(ctx, left) };
    let rr = unsafe { akar_layout_rect(ctx, right) };
    assert!((lr.w - 400.0).abs() < 1.0, "left.w = {}", lr.w);
    assert!((rr.w - 400.0).abs() < 1.0, "right.w = {}", rr.w);
    assert!((rr.x - 400.0).abs() < 1.0, "right.x = {}", rr.x);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn input_feed_does_not_panic() {
    let ctx = unsafe { akar_ctx_mock() };
    unsafe {
        akar_begin_frame(ctx, 800, 600, 1.0);
        akar_input_begin(ctx);
        akar_set_mouse_pos(ctx, 100.0, 200.0);
        akar_push_mouse_button(ctx, 0, true);
        akar_push_scroll(ctx, 0.0, -3.0);
        akar_input_end(ctx);
    }
    unsafe { akar_ctx_free(ctx) };
}
