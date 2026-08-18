use akar_core::{AkarCore, CaretMotion, Key, QuadCall, TextCall, Z_TEXT_FOREGROUND};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_edit::{
    apply_targeted_pastes, clipboard_shortcut, collapse_selection_visually, delete_selection,
    next_boundary, normalize_paste, previous_boundary, replace_selection, TextEditState,
};
use crate::AkarTheme;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TextAreaResponse {
    pub changed: bool,
    pub copy_text: Option<String>,
    pub request_paste: bool,
}

fn line_start(value: &str, position: usize) -> usize {
    value[..position].rfind('\n').map_or(0, |index| index + 1)
}

fn line_end(value: &str, position: usize) -> usize {
    value[position..]
        .find('\n')
        .map_or(value.len(), |index| position + index)
}

fn character_column(value: &str, position: usize) -> usize {
    value[line_start(value, position)..position].chars().count()
}

fn position_at_character_column(value: &str, start: usize, end: usize, column: usize) -> usize {
    value[start..end]
        .char_indices()
        .nth(column)
        .map_or(end, |(index, _)| start + index)
}

fn move_vertical(value: &str, position: usize, direction: isize) -> usize {
    let current_start = line_start(value, position);
    let column = character_column(value, position);

    if direction < 0 {
        if current_start == 0 {
            return 0;
        }
        let target_end = current_start - 1;
        let target_start = line_start(value, target_end);
        position_at_character_column(value, target_start, target_end, column)
    } else {
        let current_end = line_end(value, position);
        if current_end == value.len() {
            return value.len();
        }
        let target_start = current_end + 1;
        let target_end = line_end(value, target_start);
        position_at_character_column(value, target_start, target_end, column)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn textarea(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: &mut String,
    edit_state: &mut TextEditState,
    scroll_y: &mut f32,
    placeholder: &str,
    cursor_visible: bool,
    theme: &AkarTheme,
) -> TextAreaResponse {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return TextAreaResponse::default();
    }

    let id_u64 = layout.widget_id(node_id);
    let line_height = theme.font_size_base * 1.2;

    if core.input.is_clicked(rect) {
        core.input.focused_id = Some(id_u64);
    }

    if core.input.focused_id == Some(id_u64)
        && core.input.mouse_buttons_pressed[0]
        && !core.input.is_hovering(rect)
    {
        core.input.focused_id = None;
    }

    let focused = core.input.focused_id == Some(id_u64);

    if core.input.is_hovering(rect) {
        *scroll_y -= core.input.scroll_delta.y;
    }

    let content_height = value.lines().count() as f32 * line_height + theme.padding_y * 2.0;
    let max_scroll = (content_height - rect[3]).max(0.0);
    *scroll_y = scroll_y.clamp(0.0, max_scroll);

    let max_text_width = (rect[2] - 2.0 * theme.padding_x).max(0.0);
    let metrics = glyphon::Metrics::new(theme.font_size_base, line_height);

    let mut changed = false;
    let mut copy_text = None;
    let mut request_paste = false;

    if focused {
        edit_state.normalize(value);
        changed |= apply_targeted_pastes(
            &core.input,
            core.input.focused_id,
            id_u64,
            value,
            edit_state,
            true,
        );
        let chars: String = core.input.chars.iter().collect();
        if !chars.is_empty() {
            changed |= replace_selection(value, edit_state, &normalize_paste(&chars, true));
        }

        for event in core.input.key_events.clone() {
            if core.text_edit_keybindings.matches_select_all(&event) {
                edit_state.select_all(value);
                continue;
            }
            let clipboard =
                clipboard_shortcut(&core.text_edit_keybindings, &event, value, edit_state);
            if clipboard.2 {
                if clipboard.0.is_some() {
                    copy_text = clipboard.0;
                }
                request_paste |= clipboard.1;
                continue;
            }
            if event.key == Key::Backspace {
                if edit_state.has_selection() {
                    changed |= delete_selection(value, edit_state);
                } else if edit_state.cursor > 0 {
                    let start = previous_boundary(value, edit_state.cursor);
                    edit_state.anchor = start;
                    changed |= delete_selection(value, edit_state);
                }
            } else if event.key == Key::Delete {
                if edit_state.has_selection() {
                    changed |= delete_selection(value, edit_state);
                } else if edit_state.cursor < value.len() {
                    let end = next_boundary(value, edit_state.cursor);
                    edit_state.anchor = end;
                    changed |= delete_selection(value, edit_state);
                }
            } else {
                match event.key {
                    Key::Left if edit_state.has_selection() => {
                        let buffer_id =
                            reshape_for_motion(core, id_u64, value, metrics, max_text_width);
                        collapse_selection_visually(
                            &core.text_pipeline,
                            buffer_id,
                            edit_state,
                            CaretMotion::Left,
                        );
                    }
                    Key::Left => {
                        let buffer_id =
                            reshape_for_motion(core, id_u64, value, metrics, max_text_width);
                        edit_state.cursor = core.text_pipeline.move_caret(
                            buffer_id,
                            edit_state.cursor,
                            CaretMotion::Left,
                        );
                    }
                    Key::Right if edit_state.has_selection() => {
                        let buffer_id =
                            reshape_for_motion(core, id_u64, value, metrics, max_text_width);
                        collapse_selection_visually(
                            &core.text_pipeline,
                            buffer_id,
                            edit_state,
                            CaretMotion::Right,
                        );
                    }
                    Key::Right => {
                        let buffer_id =
                            reshape_for_motion(core, id_u64, value, metrics, max_text_width);
                        edit_state.cursor = core.text_pipeline.move_caret(
                            buffer_id,
                            edit_state.cursor,
                            CaretMotion::Right,
                        );
                    }
                    Key::Up => {
                        edit_state.cursor = move_vertical(value, edit_state.cursor, -1);
                    }
                    Key::Down => {
                        edit_state.cursor = move_vertical(value, edit_state.cursor, 1);
                    }
                    Key::Home => {
                        edit_state.cursor = line_start(value, edit_state.cursor);
                    }
                    Key::End => {
                        edit_state.cursor = line_end(value, edit_state.cursor);
                    }
                    Key::Enter => {
                        changed |= replace_selection(value, edit_state, "\n");
                    }
                    Key::Escape => {
                        core.input.focused_id = None;
                    }
                    _ => {}
                }
                edit_state.anchor = edit_state.cursor;
            }
        }
    }

    if !focused {
        edit_state.normalize(value);
    }

    core.draw_list.push_scissor(rect);

    let border_color = if focused {
        theme.primary
    } else {
        theme.base_300
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(theme.base_200),
        border_color: color_to_f32(border_color),
        corner_radii: [theme.radius_field; 4],
        border_width: theme.border_width,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let text_x = rect[0] + theme.padding_x;
    let text_y = rect[1] + theme.padding_y - *scroll_y;

    let display_text = if value.is_empty() && !focused {
        placeholder
    } else {
        value.as_str()
    };
    let text_color = if value.is_empty() && !focused {
        theme.base_300
    } else {
        theme.base_content
    };

    let buffer_id = core.text_pipeline.set_text(
        Some(id_u64),
        display_text,
        metrics,
        Some(max_text_width),
        None,
        None,
    );

    let geometry = core.text_pipeline.geometry(
        buffer_id,
        display_text,
        edit_state.cursor,
        edit_state.anchor,
    );
    if focused {
        let mut selection_color = color_to_f32(theme.info);
        selection_color[3] = 0.35;
        for selection in &geometry.selection {
            if let Some(quad) = text_edit_quad(
                [
                    text_x + selection[0],
                    text_y + selection[1],
                    selection[2],
                    selection[3],
                ],
                selection_color,
                0.01,
                rect,
            ) {
                core.draw_list.push_quad(quad);
            }
        }
    }

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: text_x,
        y: text_y,
        clip: rect,
        color: color_to_f32(text_color),
        z: 0.0,
    });

    if focused && cursor_visible {
        if let Some(caret) = geometry.caret {
            if let Some(quad) = text_edit_quad(
                [text_x + caret[0], text_y + caret[1], caret[2], caret[3]],
                color_to_f32(theme.primary),
                Z_TEXT_FOREGROUND,
                rect,
            ) {
                core.draw_list.push_quad(quad);
            }
        }
    }

    core.draw_list.pop_scissor();

    TextAreaResponse {
        changed,
        copy_text,
        request_paste,
    }
}

/// Ensures the widget's stable buffer id reflects `value` as shaped right
/// now, so a same-frame `Left`/`Right` key event always drives cosmic-text's
/// `Buffer::cursor_motion` off the paragraph this keystroke actually acted
/// on — not a shape left over from the previous frame, or from an earlier
/// edit (insert/backspace/delete/Enter) processed earlier in this same
/// frame's key-event loop. `set_text` is idempotent and already runs
/// unconditionally once per frame for rendering, so re-running it here on
/// demand is the smallest lifecycle-safe way to guarantee a fresh shape at
/// the point of use.
fn reshape_for_motion(
    core: &mut AkarCore,
    id_u64: u64,
    value: &str,
    metrics: glyphon::Metrics,
    max_text_width: f32,
) -> u64 {
    core.text_pipeline.set_text(
        Some(id_u64),
        value,
        metrics,
        Some(max_text_width),
        None,
        None,
    )
}

fn text_edit_quad(rect: [f32; 4], fill: [f32; 4], z: f32, clip: [f32; 4]) -> Option<QuadCall> {
    let x = rect[0].max(clip[0]);
    let y = rect[1].max(clip[1]);
    let width = (rect[0] + rect[2]).min(clip[0] + clip[2]) - x;
    let height = (rect[1] + rect[3]).min(clip[1] + clip[3]) - y;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(QuadCall {
        rect: [x, y, width, height],
        fill,
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_returns_default() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        let mut value = String::new();
        let mut edit_state = TextEditState::default();
        let mut scroll_y = 0.0f32;

        let result = textarea(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            "Placeholder",
            true,
            &AKAR_THEME_DARK,
        );

        assert!(!result.changed);
    }

    fn focused_area(value: &str) -> (AkarCore, Layout, NodeId, String, TextEditState, f32) {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(300.0_f32),
                height: akar_layout::length(200.0_f32),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(400.0), Some(400.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let id_u64 = layout.widget_id(node_id);
        core.input.focused_id = Some(id_u64);

        let edit_state = TextEditState {
            cursor: value.len(),
            anchor: value.len(),
        };
        (core, layout, node_id, value.to_string(), edit_state, 0.0)
    }

    fn press_arrow(
        core: &mut AkarCore,
        layout: &Layout,
        node_id: NodeId,
        value: &mut String,
        edit_state: &mut TextEditState,
        scroll_y: &mut f32,
        key: Key,
    ) {
        core.input.begin_frame();
        core.input.focused_id = Some(layout.widget_id(node_id));
        core.input.push_key(key);
        textarea(
            core,
            layout,
            node_id,
            value,
            edit_state,
            scroll_y,
            "",
            true,
            &AKAR_THEME_DARK,
        );
    }

    /// Right at the end of one line must cross into the start of the next
    /// line's global byte offset, not misread the new line-local `Cursor`
    /// index as if it were still a global offset.
    #[test]
    fn right_arrow_crosses_line_separator_to_next_lines_global_offset() {
        let (mut core, layout, node_id, mut value, mut edit_state, mut scroll_y) =
            focused_area("one\ntwo");
        edit_state.cursor = 3; // end of "one", right before '\n'
        edit_state.anchor = 3;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Right,
        );

        assert_eq!(
            edit_state,
            TextEditState {
                cursor: 4,
                anchor: 4
            },
            "must land at global offset 4 (start of \"two\"), not line-local index 4"
        );

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Left,
        );
        assert_eq!(edit_state.cursor, 3);
    }

    /// Same paragraph-direction-discovery property as `text_input`, exercised
    /// through the multi-line editor: physical `Right` is a no-op at a
    /// pure-RTL line's logical start, `Left` advances. Uses real Arabic text
    /// inside a `textarea`.
    #[test]
    fn arrow_keys_follow_shaped_paragraph_direction_in_textarea() {
        let (mut core, layout, node_id, mut value, mut edit_state, mut scroll_y) =
            focused_area("مرحبا\nsecond line");
        edit_state.cursor = 0;
        edit_state.anchor = 0;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Right,
        );
        assert_eq!(
            edit_state.cursor, 0,
            "Right at the start of an RTL first line must not move logically backward"
        );

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Left,
        );
        assert_ne!(edit_state.cursor, 0);
    }

    #[test]
    fn selection_collapse_uses_line_order_in_ltr_textarea() {
        let text = "long first line\nx";
        let second_start = "long first line\n".len();
        let (mut core, layout, node_id, mut value, mut edit_state, mut scroll_y) =
            focused_area(text);
        edit_state.cursor = second_start;
        edit_state.anchor = 10;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Left,
        );
        assert_eq!(edit_state.cursor, 10);

        edit_state.cursor = 10;
        edit_state.anchor = second_start;
        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Right,
        );
        assert_eq!(edit_state.cursor, second_start);
    }

    #[test]
    fn selection_collapse_uses_line_order_in_rtl_textarea() {
        let first_end = "שלום".len();
        let second_start = first_end + 1;
        let (mut core, layout, node_id, mut value, mut edit_state, mut scroll_y) =
            focused_area("שלום\nא");
        edit_state.cursor = second_start;
        edit_state.anchor = first_end;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Left,
        );
        assert_eq!(edit_state.cursor, first_end);

        edit_state.cursor = first_end;
        edit_state.anchor = second_start;
        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Right,
        );
        assert_eq!(edit_state.cursor, second_start);
    }

    #[test]
    fn selection_collapse_remains_bidi_aware_within_one_rtl_line() {
        let len = "שלום".len();
        let (mut core, layout, node_id, mut value, mut edit_state, mut scroll_y) =
            focused_area("שלום");
        edit_state.cursor = len;
        edit_state.anchor = 0;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Left,
        );
        assert_eq!(edit_state.cursor, len);

        edit_state.cursor = len;
        edit_state.anchor = 0;
        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            &mut scroll_y,
            Key::Right,
        );
        assert_eq!(edit_state.cursor, 0);
    }

    #[test]
    fn vertical_navigation_uses_unicode_character_columns() {
        let value = "aé🙂z\n12345\né🙂";

        assert_eq!(move_vertical(value, 7, 1), 12);
        assert_eq!(move_vertical(value, 12, 1), value.len());
        assert_eq!(move_vertical(value, value.len(), -1), 11);
        assert!(value.is_char_boundary(move_vertical(value, 7, 1)));
        assert!(value.is_char_boundary(move_vertical(value, 12, -1)));
    }

    #[test]
    fn vertical_navigation_clamps_at_document_and_short_line_edges() {
        let value = "abc\né\nwxyz";

        assert_eq!(move_vertical(value, 2, -1), 0);
        assert_eq!(move_vertical(value, 2, 1), 6);
        assert_eq!(move_vertical(value, 6, 1), 8);
        assert_eq!(move_vertical(value, value.len(), 1), value.len());
    }

    #[test]
    fn scrolled_text_edit_geometry_is_clipped_to_viewport() {
        let quad = text_edit_quad(
            [10.0, -15.0, 12.0, 20.0],
            [1.0; 4],
            0.01,
            [0.0, 0.0, 100.0, 40.0],
        )
        .expect("partially visible");
        assert_eq!(quad.rect, [10.0, 0.0, 12.0, 5.0]);
    }
}
