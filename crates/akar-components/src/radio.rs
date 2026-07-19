use akar_core::{AkarCore, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::{color_to_f32, scale_color};
use crate::AkarTheme;

pub fn radio_group(
    core: &mut AkarCore,
    layout: &Layout,
    nodes: &[NodeId],
    labels: &[&str],
    selected: &mut usize,
    theme: &AkarTheme,
) -> bool {
    let mut changed = false;

    for (i, &node) in nodes.iter().enumerate() {
        let rect = layout.rect(node);
        if rect[2] == 0.0 || rect[3] == 0.0 {
            continue;
        }
        if core.input.is_clicked(rect) {
            *selected = i;
            changed = true;
        }
    }

    for (i, (&node, &label)) in nodes.iter().zip(labels.iter()).enumerate() {
        let rect = layout.rect(node);
        if rect[2] == 0.0 || rect[3] == 0.0 {
            continue;
        }

        let hovered = core.input.is_hovering(rect);

        let circle_size = 16.0;
        let inner_size = 8.0;
        let circle_x = rect[0];
        let circle_y = rect[1] + (rect[3] - circle_size) * 0.5;
        let circle_rect = [circle_x, circle_y, circle_size, circle_size];

        let border_color = if hovered {
            scale_color(theme.base_300, 1.2)
        } else {
            theme.base_300
        };

        core.draw_list.push_quad(QuadCall {
            rect: circle_rect,
            fill: [0.0; 4],
            border_color: color_to_f32(border_color),
            corner_radii: [circle_size * 0.5; 4],
            border_width: theme.border_width,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        if i == *selected {
            let inner_x = circle_x + (circle_size - inner_size) * 0.5;
            let inner_y = circle_y + (circle_size - inner_size) * 0.5;
            let inner_rect = [inner_x, inner_y, inner_size, inner_size];
            core.draw_list.push_quad(QuadCall {
                rect: inner_rect,
                fill: color_to_f32(theme.primary),
                border_color: [0.0; 4],
                corner_radii: [inner_size * 0.5; 4],
                border_width: 0.0,
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }

        let label_buffer = core.text_pipeline.set_text(
            Some(node.into()),
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
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_nodes_are_skipped() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        let mut selected = 0usize;

        let result = radio_group(
            &mut core,
            &layout,
            &[node_id],
            &["Test"],
            &mut selected,
            &AKAR_THEME_DARK,
        );

        assert!(!result);
    }
}
