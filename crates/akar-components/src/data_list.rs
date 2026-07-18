use akar_core::AkarCore;
use akar_layout::{Layout, NodeId};

pub struct DataListState {
    pub scroll_y: f32,
}

pub struct DataListResponse {
    pub viewport_rect: [f32; 4],
    pub content_origin: [f32; 2],
    pub visible_range: std::ops::Range<usize>,
    pub visible_keys: Vec<u64>,
}

/// Begins a fixed-height virtualized list scope.
///
/// All items must share the same `item_height`. Variable-height virtualization
/// is out of scope; applications may use an unvirtualized layout list or an
/// application-managed measurement/index until a dedicated follow-up is designed.
///
/// The caller must supply a stable per-item `keys` slice (record identity) so
/// that `widget_id_keyed(node, key)` can be used for focusable children inside
/// each visible item. Using plain `widget_id(node)` would tie widget identity
/// to the screen row position and corrupt focus/text-buffer state on scroll
/// (see ADR-016a).
pub fn data_list_begin(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    state: &mut DataListState,
    item_count: usize,
    item_height: f32,
    keys: &[u64],
) -> DataListResponse {
    let rect = layout.rect(node);
    let [_, y, _, h] = rect;

    if core.input.is_hovering(rect) {
        state.scroll_y -= core.input.scroll_delta.y;
    }

    let content_height = item_count as f32 * item_height;
    let max_scroll = (content_height - h).max(0.0);
    state.scroll_y = state.scroll_y.clamp(0.0, max_scroll);

    core.draw_list.push_scissor(rect);

    let visible_range = akar_core::list_clip(item_count, item_height, state.scroll_y, h);
    let visible_keys: Vec<u64> = keys.get(visible_range.clone()).unwrap_or(&[]).to_vec();

    DataListResponse {
        viewport_rect: rect,
        content_origin: [0.0, y - state.scroll_y],
        visible_range,
        visible_keys,
    }
}

pub fn data_list_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_layout::{length, Size, Style};

    fn make_list_layout(viewport_h: f32) -> (Layout, NodeId) {
        let mut layout = Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(200.0),
                height: length(viewport_h),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[node]);
        layout.compute(root, (Some(400.0), Some(800.0)), |_, _, _, _, _| Size::ZERO);
        (layout, node)
    }

    fn make_keys(n: usize) -> Vec<u64> {
        (0..n as u64).map(|i| (i + 1) * 1000).collect()
    }

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(10);
        let mut state = DataListState { scroll_y: 0.0 };

        data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        assert!(core.draw_list.active_scissor().is_some());

        data_list_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }

    #[test]
    fn scroll_y_clamped_to_zero() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(10);
        let mut state = DataListState { scroll_y: -50.0 };

        data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        data_list_end(&mut core);

        assert_eq!(state.scroll_y, 0.0);
    }

    #[test]
    fn scroll_y_clamped_to_max() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(10);
        let mut state = DataListState { scroll_y: 9999.0 };

        data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        data_list_end(&mut core);

        assert_eq!(state.scroll_y, 300.0);
    }

    #[test]
    fn visible_range_covers_viewport() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(20);
        let mut state = DataListState { scroll_y: 0.0 };

        let resp = data_list_begin(&mut core, &layout, node, &mut state, 20, 50.0, &keys);
        data_list_end(&mut core);

        assert!(resp.visible_range.start <= 1);
        assert!(resp.visible_range.end >= 4);
    }

    #[test]
    fn visible_keys_match_range() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(10);
        let mut state = DataListState { scroll_y: 0.0 };

        let resp = data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        data_list_end(&mut core);

        let expected: Vec<u64> = keys[resp.visible_range.clone()].to_vec();
        assert_eq!(resp.visible_keys, expected);
    }

    #[test]
    fn empty_list_returns_empty_range() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys: Vec<u64> = vec![];
        let mut state = DataListState { scroll_y: 0.0 };

        let resp = data_list_begin(&mut core, &layout, node, &mut state, 0, 50.0, &keys);
        data_list_end(&mut core);

        assert_eq!(resp.visible_range, 0..0);
        assert!(resp.visible_keys.is_empty());
    }

    #[test]
    fn keys_slice_shorter_than_items() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = vec![1000u64, 2000, 3000];
        let mut state = DataListState { scroll_y: 0.0 };

        let resp = data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        data_list_end(&mut core);

        assert!(resp.visible_keys.len() <= 3);
    }

    #[test]
    fn scrolled_visible_keys_offset() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(20);
        let mut state = DataListState { scroll_y: 250.0 };

        let resp = data_list_begin(&mut core, &layout, node, &mut state, 20, 50.0, &keys);
        data_list_end(&mut core);

        for &k in &resp.visible_keys {
            let idx = keys.iter().position(|&x| x == k).unwrap();
            assert!(resp.visible_range.contains(&idx));
        }
    }

    #[test]
    fn nested_scissor_intersection_and_restore() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let outer_rect = [10.0, 10.0, 300.0, 300.0];
        core.draw_list.push_scissor(outer_rect);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(10);
        let mut state = DataListState { scroll_y: 0.0 };

        data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);

        let inner = core.draw_list.active_scissor().unwrap();
        let list_rect = layout.rect(node);
        let expected_x = outer_rect[0].max(list_rect[0]);
        let expected_y = outer_rect[1].max(list_rect[1]);
        let expected_w =
            (outer_rect[0] + outer_rect[2]).min(list_rect[0] + list_rect[2]) - expected_x;
        let expected_h =
            (outer_rect[1] + outer_rect[3]).min(list_rect[1] + list_rect[3]) - expected_y;
        assert_eq!(inner, [expected_x, expected_y, expected_w, expected_h]);

        data_list_end(&mut core);

        let restored = core.draw_list.active_scissor().unwrap();
        assert_eq!(restored, [10.0, 10.0, 300.0, 300.0]);

        core.draw_list.pop_scissor();
        assert!(core.draw_list.active_scissor().is_none());
    }

    #[test]
    fn adr016a_scroll_does_not_transfer_focus() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(20);
        let mut state = DataListState { scroll_y: 0.0 };

        let item_height = 50.0;
        data_list_begin(&mut core, &layout, node, &mut state, 20, item_height, &keys);
        let first_visible_idx = (state.scroll_y / item_height) as usize;
        let first_key = keys[first_visible_idx];
        let first_item_id = layout.widget_id_keyed(node, first_key);
        core.input.focused_id = Some(first_item_id);
        data_list_end(&mut core);

        state.scroll_y = item_height;
        core.draw_list.begin_frame(1.0);
        data_list_begin(&mut core, &layout, node, &mut state, 20, item_height, &keys);
        let new_first_visible_idx = (state.scroll_y / item_height) as usize;
        let new_first_key = keys[new_first_visible_idx];
        let new_first_item_id = layout.widget_id_keyed(node, new_first_key);
        data_list_end(&mut core);

        assert_ne!(first_key, new_first_key);
        assert_ne!(core.input.focused_id, Some(new_first_item_id));
    }

    #[test]
    fn scroll_consumed_only_when_hovering_inside() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_list_layout(200.0);
        let keys = make_keys(10);
        let mut state = DataListState { scroll_y: 0.0 };
        let rect = layout.rect(node);

        core.draw_list.begin_frame(1.0);
        core.input.set_mouse_pos(rect[0] - 10.0, rect[1] + 10.0);
        core.input.push_scroll(0.0, -50.0);
        data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        data_list_end(&mut core);
        assert_eq!(state.scroll_y, 0.0);

        core.input.begin_frame();
        core.draw_list.begin_frame(1.0);
        core.input.set_mouse_pos(rect[0] + 10.0, rect[1] + 10.0);
        core.input.push_scroll(0.0, -50.0);
        data_list_begin(&mut core, &layout, node, &mut state, 10, 50.0, &keys);
        data_list_end(&mut core);
        assert_eq!(state.scroll_y, 50.0);
    }
}
