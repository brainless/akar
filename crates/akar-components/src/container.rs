use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::box_style::BoxStyle;
use crate::color::color_to_f32;

pub fn container(core: &mut AkarCore, layout: &Layout, node_id: NodeId, style: &BoxStyle) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 || style.fill == 0 {
        return;
    }

    let (shadow_color, shadow_offset, shadow_blur, shadow_spread) = match &style.shadow {
        Some(s) => (color_to_f32(s.color), s.offset, s.blur, s.spread),
        None => ([0.0; 4], [0.0; 2], 0.0, 0.0),
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(style.fill),
        border_color: color_to_f32(style.border_color),
        corner_radii: style.corner_radii,
        border_width: style.border_width,
        z: 0.0,
        shadow_blur,
        shadow_spread,
        shadow_color,
        shadow_offset,
        _pad: [0.0; 2],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_style::BoxStyle;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> akar_layout::NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(100.0),
                height: akar_layout::length(100.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(200.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn transparent_fill_pushes_no_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();
        container(&mut core, &layout, node, &BoxStyle::flat(0x00000000));
        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn solid_fill_pushes_one_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();
        container(&mut core, &layout, node, &BoxStyle::flat(0xFF0000FF));
        assert_eq!(core.draw_list.sorted_quads().len(), 1);
    }

    #[test]
    fn zero_area_pushes_no_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();
        container(&mut core, &layout, node, &BoxStyle::flat(0xFF0000FF));
        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn shadow_fields_propagate_to_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();
        let style = BoxStyle {
            fill: 0xFFFFFFFF,
            border_color: 0,
            border_width: 0.0,
            corner_radii: [0.0; 4],
            shadow: Some(crate::box_style::BoxShadow {
                color: 0x00000080,
                offset: [2.0, 4.0],
                blur: 8.0,
                spread: 0.0,
            }),
        };
        container(&mut core, &layout, node, &style);
        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 1);
        assert!(quads[0].shadow_blur > 0.0);
        assert!(quads[0].shadow_color[3] > 0.0);
    }
}
