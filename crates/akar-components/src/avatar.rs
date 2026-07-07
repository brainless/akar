use akar_core::{AkarCore, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

fn derive_color_idx(initials: &str) -> usize {
    let chars: Vec<char> = initials.chars().collect();
    let c0 = chars.first().copied().unwrap_or('A') as u32;
    let c1 = chars
        .get(1)
        .copied()
        .unwrap_or(chars.first().copied().unwrap_or('A')) as u32;
    ((c0 * 7 + c1 * 31) % 6) as usize
}

pub fn avatar(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    initials: &str,
    color: Option<u32>,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let diameter = rect[2].min(rect[3]);
    let fill = match color {
        Some(c) => c,
        None => {
            let palette = [
                theme.primary,
                theme.secondary,
                theme.accent,
                theme.info,
                theme.success,
                theme.warning,
            ];
            palette[derive_color_idx(initials)]
        }
    };

    let cx = rect[0] + rect[2] / 2.0;
    let cy = rect[1] + rect[3] / 2.0;
    let circle_rect = [cx - diameter / 2.0, cy - diameter / 2.0, diameter, diameter];

    core.draw_list.push_quad(QuadCall {
        rect: circle_rect,
        fill: color_to_f32(fill),
        border_color: [0.0; 4],
        corner_radii: [diameter / 2.0; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let display: String = initials.chars().take(2).collect();
    if display.is_empty() {
        return;
    }

    let font_size = theme.font_size_sm * 0.8;
    let line_height = font_size * 1.2;

    let buf_id = core.text_pipeline.set_text(
        Some(node_id.into()),
        &display,
        glyphon::Metrics::new(font_size, line_height),
        Some(diameter),
        None,
    );

    let text_size = core.text_pipeline.measure(buf_id, Some(diameter));
    let text_x = cx - text_size.x / 2.0;
    let text_y = cy - text_size.y / 2.0;

    core.draw_list.push_text(TextCall {
        buffer_id: buf_id,
        x: text_x,
        y: text_y,
        clip: rect,
        color: color_to_f32(theme.base_content),
        z: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_core::DrawCall;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout, size: f32) -> akar_layout::NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(size),
                height: akar_layout::length(size),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(size), Some(size)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn zero_area_does_nothing() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();
        avatar(&mut core, &layout, node_id, "AB", None, &AKAR_THEME_DARK);
        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn initials_text_is_rendered() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout, 48.0);
        let mut core = AkarCore::mock();
        avatar(&mut core, &layout, node, "AB", None, &AKAR_THEME_DARK);

        let text_count = core
            .draw_list
            .text_calls()
            .iter()
            .filter(|c| matches!(c, DrawCall::Text(_)))
            .count();
        assert_eq!(text_count, 1);
    }

    #[test]
    fn color_is_deterministic() {
        let mut layout = akar_layout::Layout::new();
        let node1 = sized_node(&mut layout, 48.0);
        let node2 = sized_node(&mut layout, 48.0);
        let mut core1 = AkarCore::mock();
        let mut core2 = AkarCore::mock();
        avatar(&mut core1, &layout, node1, "XY", None, &AKAR_THEME_DARK);
        avatar(&mut core2, &layout, node2, "XY", None, &AKAR_THEME_DARK);

        let q1 = core1.draw_list.sorted_quads();
        let q2 = core2.draw_list.sorted_quads();
        assert_eq!(q1[0].fill, q2[0].fill);
    }

    #[test]
    fn explicit_color_used() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout, 48.0);
        let mut core = AkarCore::mock();
        avatar(
            &mut core,
            &layout,
            node,
            "AB",
            Some(0xFF00FFFF),
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads[0].fill, [1.0, 0.0, 1.0, 1.0]);
    }
}
