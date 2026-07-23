use akar_core::{AkarCore, Key, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_edit::{
    delete_selection, next_boundary, normalize_paste, previous_boundary, replace_selection,
    TextEditState,
};
use crate::AkarTheme;

pub struct TextInputResponse {
    pub changed: bool,
    pub submitted: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn text_input(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: &mut String,
    edit_state: &mut TextEditState,
    placeholder: &str,
    cursor_visible: bool,
    theme: &AkarTheme,
) -> TextInputResponse {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return TextInputResponse {
            changed: false,
            submitted: false,
        };
    }

    let id_u64 = layout.widget_id(node_id);

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

    let mut changed = false;
    let mut submitted = false;

    if focused {
        edit_state.normalize(value);
        let chars: String = core.input.chars.iter().collect();
        if !chars.is_empty() {
            changed |= replace_selection(value, edit_state, &normalize_paste(&chars, false));
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
                    Key::Left => {
                        edit_state.cursor = previous_boundary(value, edit_state.cursor);
                        edit_state.anchor = edit_state.cursor;
                    }
                    Key::Right if edit_state.has_selection() => edit_state.collapse_to_end(),
                    Key::Right => {
                        edit_state.cursor = next_boundary(value, edit_state.cursor);
                        edit_state.anchor = edit_state.cursor;
                    }
                    Key::Home => {
                        edit_state.cursor = 0;
                        edit_state.anchor = 0;
                    }
                    Key::End => {
                        edit_state.cursor = value.len();
                        edit_state.anchor = value.len();
                    }
                    Key::Enter => {
                        submitted = true;
                    }
                    Key::Escape => {
                        core.input.focused_id = None;
                    }
                    _ => {}
                }
            }
        }
    }

    if !focused {
        edit_state.normalize(value);
    }

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
    let text_y = rect[1] + theme.padding_y;
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
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
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
        let cursor_x = text_x + edit_state.cursor as f32 * theme.font_size_base * 0.5;
        let cursor_y = rect[1] + theme.padding_y;
        let cursor_height = theme.font_size_base * 1.2;

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

    TextInputResponse { changed, submitted }
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

        let result = text_input(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            "Placeholder",
            true,
            &AKAR_THEME_DARK,
        );

        assert!(!result.changed);
        assert!(!result.submitted);
    }
}
