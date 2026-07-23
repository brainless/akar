use akar_core::{AkarCore, Key, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_edit::{
    delete_selection, next_boundary, normalize_paste, previous_boundary, replace_selection,
    TextEditState,
};
use crate::AkarTheme;

pub struct TextAreaResponse {
    pub changed: bool,
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
        return TextAreaResponse { changed: false };
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

    let mut changed = false;

    if focused {
        edit_state.normalize(value);
        let chars: String = core.input.chars.iter().collect();
        if !chars.is_empty() {
            changed |= replace_selection(value, edit_state, &normalize_paste(&chars, true));
        }

        for event in core.input.key_events.clone() {
            if core.text_edit_keybindings.matches_select_all(&event) {
                edit_state.select_all(value);
            } else if event.key == Key::Backspace {
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
                    Key::Left if edit_state.has_selection() => edit_state.collapse_to_start(),
                    Key::Left => edit_state.cursor = previous_boundary(value, edit_state.cursor),
                    Key::Right if edit_state.has_selection() => edit_state.collapse_to_end(),
                    Key::Right => edit_state.cursor = next_boundary(value, edit_state.cursor),
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
    let max_text_width = rect[2] - 2.0 * theme.padding_x;

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
        Some(layout.widget_id(node_id)),
        display_text,
        glyphon::Metrics::new(theme.font_size_base, line_height),
        Some(max_text_width.max(0.0)),
        None,
    );

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: text_x,
        y: text_y,
        clip: rect,
        color: color_to_f32(text_color),
        z: 0.0,
    });

    if focused && cursor_visible {
        let line = value[..edit_state.cursor].matches('\n').count();
        let line_start = value[..edit_state.cursor]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let col = edit_state.cursor - line_start;
        let cursor_x = text_x + col as f32 * theme.font_size_base * 0.5;
        let cursor_y = rect[1] + theme.padding_y + line as f32 * line_height - *scroll_y;
        let cursor_height = line_height;

        core.draw_list.push_quad(QuadCall {
            rect: [cursor_x, cursor_y, 2.0, cursor_height],
            fill: color_to_f32(theme.primary),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    core.draw_list.pop_scissor();

    TextAreaResponse { changed }
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
}
