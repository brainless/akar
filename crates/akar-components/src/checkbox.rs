use akar_core::{AkarCore, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::{color_to_f32, scale_color};
use crate::AkarTheme;

pub fn checkbox(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    checked: &mut bool,
    label: &str,
    theme: &AkarTheme,
) -> bool {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return false;
    }

    let hovered = core.input.is_hovering(rect);
    let clicked = core.input.is_clicked(rect);

    let box_size = 18.0;
    let box_x = rect[0];
    let box_y = rect[1] + (rect[3] - box_size) * 0.5;
    let box_rect = [box_x, box_y, box_size, box_size];

    let fill = if *checked { theme.primary } else { 0x00000000 };
    let border_color = if hovered {
        scale_color(theme.base_300, 1.2)
    } else {
        theme.base_300
    };

    core.draw_list.push_quad(QuadCall {
        rect: box_rect,
        fill: color_to_f32(fill),
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

    if *checked {
        let check_buffer = core.text_pipeline.set_text(
            Some(layout.widget_id(node_id)),
            "\u{2713}",
            glyphon::Metrics::new(theme.font_size_sm, theme.font_size_sm * 1.2),
            Some(box_size),
            None,
        );
        core.draw_list.push_text(TextCall {
            buffer_id: check_buffer,
            x: box_x,
            y: box_y,
            clip: box_rect,
            color: color_to_f32(theme.primary_content),
            z: 0.0,
        });
    }

    let label_buffer = core.text_pipeline.set_text(
        Some(layout.widget_id(node_id) + 1),
        label,
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
        Some(rect[2] - 24.0),
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: label_buffer,
        x: rect[0] + 24.0,
        y: rect[1] + theme.padding_y,
        clip: rect,
        color: color_to_f32(theme.base_content),
        z: 0.0,
    });

    if clicked {
        *checked = !*checked;
        return true;
    }

    false
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
        let mut checked = false;

        let result = checkbox(
            &mut core,
            &layout,
            node_id,
            &mut checked,
            "Test",
            &AKAR_THEME_DARK,
        );

        assert!(!result);
        assert!(!checked);
    }
}
