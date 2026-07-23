use akar_c_api::{
    akar_add_child, akar_begin_frame, akar_ctx_free, akar_ctx_mock, akar_data_item,
    akar_data_item_style_default, akar_data_list_begin, akar_data_list_end, akar_input_begin,
    akar_input_end, akar_layout_compute, akar_layout_rect, akar_new_fixed_leaf, akar_new_flex_col,
    akar_new_flex_row, akar_new_leaf, akar_push_key_event, akar_push_mouse_button, akar_push_paste,
    akar_push_scroll, akar_set_mouse_pos, akar_set_text_edit_keybindings, akar_text_input,
    AkarDataItemStyle, AkarDataListState, AkarShortcut, AkarTextEditKeybindings, AkarTextEditState,
    AKAR_KEY_CHARACTER, AKAR_SHORTCUT_MODIFIER_ALT,
};
use std::ffi::CString;

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

unsafe fn text_input_fixture() -> (*mut akar_c_api::AkarCtx, u64) {
    let ctx = unsafe { akar_ctx_mock() };
    unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
    let node = unsafe { akar_new_fixed_leaf(ctx, 240.0, 40.0) };
    let root = unsafe { akar_new_flex_col(ctx) };
    unsafe { akar_add_child(ctx, root, node) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };
    (ctx, node)
}

unsafe fn focus_text_input(
    ctx: *mut akar_c_api::AkarCtx,
    node: u64,
    value: &mut [u8],
    value_len: &mut u32,
    state: &mut AkarTextEditState,
) {
    let placeholder = CString::new("").unwrap();
    unsafe {
        akar_input_begin(ctx);
        akar_set_mouse_pos(ctx, 10.0, 10.0);
        akar_push_mouse_button(ctx, 0, true);
        akar_push_mouse_button(ctx, 0, false);
        let _ = akar_text_input(
            ctx,
            node,
            value.as_mut_ptr(),
            value_len,
            value.len() as u32,
            state,
            placeholder.as_ptr(),
            true,
            std::ptr::null_mut(),
            0,
        );
    }
}

fn shortcut(character: char) -> AkarShortcut {
    AkarShortcut {
        modifiers: AKAR_SHORTCUT_MODIFIER_ALT,
        key: AKAR_KEY_CHARACTER,
        character: character as u32,
    }
}

#[test]
fn text_input_custom_bindings_select_copy_and_request_paste() {
    let (ctx, node) = unsafe { text_input_fixture() };
    let bindings = AkarTextEditKeybindings {
        select_all: shortcut('x'),
        copy: shortcut('y'),
        paste: shortcut('z'),
    };
    assert!(unsafe { akar_set_text_edit_keybindings(ctx, bindings) });

    let mut value = [0u8; 32];
    value[..5].copy_from_slice(b"hello");
    let mut value_len = 5;
    let mut state = AkarTextEditState {
        cursor: 5,
        anchor: 5,
    };
    unsafe { focus_text_input(ctx, node, &mut value, &mut value_len, &mut state) };

    let placeholder = CString::new("").unwrap();
    unsafe {
        akar_input_begin(ctx);
        akar_push_key_event(
            ctx,
            AKAR_KEY_CHARACTER,
            'x' as u32,
            AKAR_SHORTCUT_MODIFIER_ALT,
            false,
        );
    }
    let selected = unsafe {
        akar_text_input(
            ctx,
            node,
            value.as_mut_ptr(),
            &mut value_len,
            value.len() as u32,
            &mut state,
            placeholder.as_ptr(),
            true,
            std::ptr::null_mut(),
            0,
        )
    };
    assert_eq!(selected.edit_state.anchor, 0);
    assert_eq!(selected.edit_state.cursor, 5);

    let mut copied = [0u8; 8];
    unsafe {
        akar_input_begin(ctx);
        akar_push_key_event(
            ctx,
            AKAR_KEY_CHARACTER,
            'y' as u32,
            AKAR_SHORTCUT_MODIFIER_ALT,
            false,
        );
    }
    let copied_response = unsafe {
        akar_text_input(
            ctx,
            node,
            value.as_mut_ptr(),
            &mut value_len,
            value.len() as u32,
            &mut state,
            placeholder.as_ptr(),
            true,
            copied.as_mut_ptr(),
            copied.len() as u32,
        )
    };
    assert_eq!(copied_response.copy_len, 5);
    assert_eq!(copied_response.copy_required_len, 5);
    assert_eq!(&copied[..5], b"hello");
    assert_eq!(copied[5], 0);

    unsafe {
        akar_input_begin(ctx);
        akar_push_key_event(
            ctx,
            AKAR_KEY_CHARACTER,
            'z' as u32,
            AKAR_SHORTCUT_MODIFIER_ALT,
            false,
        );
    }
    let paste_response = unsafe {
        akar_text_input(
            ctx,
            node,
            value.as_mut_ptr(),
            &mut value_len,
            value.len() as u32,
            &mut state,
            placeholder.as_ptr(),
            true,
            std::ptr::null_mut(),
            0,
        )
    };
    assert!(paste_response.request_paste);
    assert_eq!(paste_response.widget_id, node);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn text_input_targeted_paste_respects_utf8_capacity() {
    let (ctx, node) = unsafe { text_input_fixture() };
    let mut value = [0u8; 5];
    value[0] = b'a';
    let mut value_len = 1;
    let mut state = AkarTextEditState {
        cursor: 1,
        anchor: 0,
    };
    unsafe { focus_text_input(ctx, node, &mut value, &mut value_len, &mut state) };

    let paste = "ééé";
    unsafe {
        akar_input_begin(ctx);
        assert!(akar_push_paste(
            ctx,
            node,
            paste.as_ptr(),
            paste.len() as u32
        ));
    }
    let placeholder = CString::new("").unwrap();
    let response = unsafe {
        akar_text_input(
            ctx,
            node,
            value.as_mut_ptr(),
            &mut value_len,
            value.len() as u32,
            &mut state,
            placeholder.as_ptr(),
            true,
            std::ptr::null_mut(),
            0,
        )
    };
    assert!(response.changed);
    assert_eq!(value_len, 4);
    assert_eq!(
        std::str::from_utf8(&value[..value_len as usize]).unwrap(),
        "éé"
    );
    assert_eq!(value[4], 0);
    assert_eq!(state.cursor, 4);
    assert_eq!(state.anchor, 4);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn text_input_copy_without_selection_writes_nothing() {
    let (ctx, node) = unsafe { text_input_fixture() };
    let bindings = AkarTextEditKeybindings {
        select_all: shortcut('x'),
        copy: shortcut('y'),
        paste: shortcut('z'),
    };
    assert!(unsafe { akar_set_text_edit_keybindings(ctx, bindings) });

    let mut value = [0u8; 16];
    value[..5].copy_from_slice(b"hello");
    let mut value_len = 5;
    let mut state = AkarTextEditState {
        cursor: 5,
        anchor: 5,
    };
    unsafe { focus_text_input(ctx, node, &mut value, &mut value_len, &mut state) };

    let mut copied = [0x55u8; 8];
    unsafe {
        akar_input_begin(ctx);
        akar_push_key_event(
            ctx,
            AKAR_KEY_CHARACTER,
            'y' as u32,
            AKAR_SHORTCUT_MODIFIER_ALT,
            false,
        );
    }
    let placeholder = CString::new("").unwrap();
    let response = unsafe {
        akar_text_input(
            ctx,
            node,
            value.as_mut_ptr(),
            &mut value_len,
            value.len() as u32,
            &mut state,
            placeholder.as_ptr(),
            true,
            copied.as_mut_ptr(),
            copied.len() as u32,
        )
    };
    assert_eq!(response.copy_len, 0);
    assert_eq!(response.copy_required_len, 0);
    assert_eq!(copied, [0x55; 8]);

    unsafe { akar_ctx_free(ctx) };
}
