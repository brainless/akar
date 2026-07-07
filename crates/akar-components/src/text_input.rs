use akar_core::{AkarCore, Key, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub struct TextInputResponse {
    pub changed: bool,
    pub submitted: bool,
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut i = pos.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    s[..i]
        .char_indices()
        .next_back()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut i = pos.min(s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    if i >= s.len() {
        return s.len();
    }
    let c = s[i..].chars().next().unwrap();
    i + c.len_utf8()
}

#[allow(clippy::too_many_arguments)]
pub fn text_input(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: &mut String,
    cursor_pos: &mut usize,
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

    let id_u64 = u64::from(node_id);

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
        for &c in &core.input.chars {
            value.insert(*cursor_pos, c);
            *cursor_pos += 1;
            changed = true;
        }

        for key in &core.input.keys_pressed {
            match key {
                Key::Backspace if *cursor_pos > 0 => {
                    let len = value[..*cursor_pos]
                        .chars()
                        .last()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    if len > 0 {
                        value.drain(*cursor_pos - len..*cursor_pos);
                        *cursor_pos -= len;
                        changed = true;
                    }
                }
                Key::Delete if *cursor_pos < value.len() => {
                    let len = value[*cursor_pos..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    if len > 0 {
                        value.drain(*cursor_pos..*cursor_pos + len);
                        changed = true;
                    }
                }
                Key::Left if *cursor_pos > 0 => {
                    *cursor_pos = prev_char_boundary(value, *cursor_pos);
                }
                Key::Right if *cursor_pos < value.len() => {
                    *cursor_pos = next_char_boundary(value, *cursor_pos);
                }
                Key::Home => {
                    *cursor_pos = 0;
                }
                Key::End => {
                    *cursor_pos = value.len();
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
        Some(node_id.into()),
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
        let cursor_x = text_x + *cursor_pos as f32 * theme.font_size_base * 0.5;
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
        let mut cursor_pos = 0usize;

        let result = text_input(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut cursor_pos,
            "Placeholder",
            true,
            &AKAR_THEME_DARK,
        );

        assert!(!result.changed);
        assert!(!result.submitted);
    }
}
