use akar_c_api::{
    akar_add_child, akar_begin_frame, akar_ctx_free, akar_ctx_mock, akar_data_item,
    akar_data_item_style_default, akar_data_list_begin, akar_data_list_end, akar_input_begin,
    akar_input_end, akar_layout_compute, akar_layout_rect, akar_new_fixed_leaf, akar_new_flex_col,
    akar_new_flex_row, akar_new_leaf, akar_push_mouse_button, akar_push_scroll, akar_set_mouse_pos,
    AkarDataItemStyle, AkarDataListState,
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

#[test]
fn data_item_style_default_fills_valid_style() {
    let ctx = unsafe { akar_ctx_mock() };
    unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };

    let mut style = AkarDataItemStyle {
        surface: [0.0; 4],
        padding_x: 0.0,
        padding_y: 0.0,
        spacing: 0.0,
        color_normal: [0.0; 4],
        color_hover: [0.0; 4],
        color_pressed: [0.0; 4],
        color_selected: [0.0; 4],
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    };

    unsafe { akar_data_item_style_default(ctx, &mut style) };

    assert!(style.padding_x > 0.0, "padding_x should be positive");
    assert!(style.padding_y > 0.0, "padding_y should be positive");
    assert!(
        style.corner_radius >= 0.0,
        "corner_radius should be non-negative"
    );

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn data_item_returns_response_without_panic() {
    let ctx = unsafe { akar_ctx_mock() };
    unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };

    let mut style = AkarDataItemStyle {
        surface: [0.0; 4],
        padding_x: 0.0,
        padding_y: 0.0,
        spacing: 0.0,
        color_normal: [0.0; 4],
        color_hover: [0.0; 4],
        color_pressed: [0.0; 4],
        color_selected: [0.0; 4],
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    };
    unsafe { akar_data_item_style_default(ctx, &mut style) };

    let node = unsafe { akar_new_fixed_leaf(ctx, 100.0, 50.0) };
    let root = unsafe { akar_new_flex_col(ctx) };
    unsafe { akar_add_child(ctx, root, node) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let resp = unsafe { akar_data_item(ctx, node, 42, &style) };
    let _ = resp.hovered;
    let _ = resp.pressed;
    let _ = resp.clicked;

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn data_list_begin_end_pushes_and_pops_scissor() {
    let ctx = unsafe { akar_ctx_mock() };
    unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };

    let node = unsafe { akar_new_fixed_leaf(ctx, 200.0, 200.0) };
    let root = unsafe { akar_new_flex_col(ctx) };
    unsafe { akar_add_child(ctx, root, node) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let mut state = AkarDataListState { scroll_y: 0.0 };
    let keys: Vec<u64> = (0..10).map(|i| (i + 1) * 1000).collect();

    let resp = unsafe {
        akar_data_list_begin(
            ctx,
            node,
            &mut state,
            10,
            50.0,
            keys.as_ptr(),
            keys.len() as u32,
        )
    };

    assert!(
        resp.viewport_rect[2] > 0.0,
        "viewport width should be positive"
    );
    assert!(
        resp.viewport_rect[3] > 0.0,
        "viewport height should be positive"
    );
    assert!(
        resp.visible_range_end > resp.visible_range_start,
        "should have visible items"
    );

    unsafe { akar_data_list_end(ctx) };

    unsafe { akar_ctx_free(ctx) };
}
