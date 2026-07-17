use akar_core::{AkarCore, QuadCall, TextCall, Z_OVERLAY};
use akar_layout::{Layout, NodeId};

use crate::color::{color_to_f32, scale_color};
use crate::{dropdown_begin, dropdown_end, AkarTheme};

#[allow(clippy::too_many_arguments)]
pub fn select(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    options: &[&str],
    selected: &mut usize,
    open: &mut bool,
    theme: &AkarTheme,
    viewport_rect: [f32; 4],
) -> bool {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return false;
    }

    let mut changed = false;

    let item_height = 28.0;
    let num_visible = 4.min(options.len());
    let total_height = item_height * num_visible as f32;
    let mut dropdown_y = rect[1] + rect[3];
    if dropdown_y + total_height > viewport_rect[1] + viewport_rect[3] {
        dropdown_y = rect[1] - total_height;
    }
    let dropdown_rect = [rect[0], dropdown_y, rect[2], total_height];

    if *open
        && core.input.mouse_buttons_pressed[0]
        && !core.input.is_hovering(rect)
        && !core.input.is_hovering(dropdown_rect)
    {
        *open = false;
    }

    let hovered = core.input.is_hovering(rect);
    let border_color = if hovered {
        scale_color(theme.base_300, 1.2)
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

    let selected_label = if *selected < options.len() {
        options[*selected]
    } else {
        ""
    };
    let label_buffer = core.text_pipeline.set_text(
        Some(u64::from(node_id)),
        selected_label,
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
        Some(rect[2] - 24.0),
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: label_buffer,
        x: rect[0] + theme.padding_x,
        y: rect[1] + theme.padding_y,
        clip: rect,
        color: color_to_f32(theme.base_content),
        z: 0.0,
    });

    let chevron_buffer = core.text_pipeline.set_text(
        Some(u64::from(node_id).wrapping_add(1_000_000)),
        "\u{25BC}",
        glyphon::Metrics::new(theme.font_size_sm, theme.font_size_sm * 1.2),
        None,
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: chevron_buffer,
        x: rect[0] + rect[2] - 20.0,
        y: rect[1] + (rect[3] - theme.font_size_sm) * 0.5,
        clip: rect,
        color: color_to_f32(theme.base_content),
        z: 0.0,
    });

    if *open {
        let state = dropdown_begin(core, rect, item_height, viewport_rect, true, theme);

        if state.is_open {
            for (i, option) in options.iter().enumerate() {
                if i >= num_visible {
                    break;
                }
                let item_rect = [
                    state.content_rect[0],
                    state.content_rect[1] + i as f32 * item_height,
                    state.content_rect[2],
                    item_height,
                ];

                let item_hovered = core.input.is_hovering(item_rect);
                let item_clicked = core.input.is_clicked(item_rect);

                if item_clicked {
                    *selected = i;
                    *open = false;
                    changed = true;
                }

                if item_hovered {
                    core.draw_list.push_quad(QuadCall {
                        rect: item_rect,
                        fill: color_to_f32(theme.base_300),
                        border_color: [0.0; 4],
                        corner_radii: [0.0; 4],
                        border_width: 0.0,
                        z: Z_OVERLAY,
                        shadow_blur: 0.0,
                        shadow_spread: 0.0,
                        shadow_color: [0.0; 4],
                        shadow_offset: [0.0; 2],
                        _pad: [0.0; 2],
                    });
                }

                let option_buffer = core.text_pipeline.set_text(
                    Some(u64::from(node_id).wrapping_add(1_000_001 + i as u64)),
                    option,
                    glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
                    Some(state.content_rect[2]),
                    None,
                );
                core.draw_list.push_text(TextCall {
                    buffer_id: option_buffer,
                    x: item_rect[0] + 4.0,
                    y: item_rect[1] + (item_height - theme.font_size_base) * 0.5,
                    clip: item_rect,
                    color: color_to_f32(theme.base_content),
                    z: Z_OVERLAY,
                });
            }
        }

        dropdown_end(core);
    }

    let toggle_clicked = core.input.is_clicked(rect);
    if toggle_clicked {
        *open = !*open;
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_returns_false() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        let mut selected = 0usize;
        let mut open = false;

        let result = select(
            &mut core,
            &layout,
            node_id,
            &["A", "B", "C"],
            &mut selected,
            &mut open,
            &AKAR_THEME_DARK,
            [0.0, 0.0, 800.0, 600.0],
        );

        assert!(!result);
        assert!(!open);
    }
}
