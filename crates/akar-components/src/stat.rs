use akar_core::{AkarCore, TextCall};
use akar_layout::{Layout, NodeId};

use crate::box_style::BoxStyle;
use crate::color::color_to_f32;
use crate::container::container;
use crate::AkarTheme;

pub fn stat(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    title: &str,
    value: &str,
    description: Option<&str>,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    container(core, layout, node_id, &BoxStyle::card(theme));

    let [x, y, w, h] = rect;
    let text_max_w = w - 2.0 * theme.padding_x;
    if text_max_w <= 0.0 {
        return;
    }

    let key = layout
        .widget_id(node_id)
        .wrapping_mul(3)
        .wrapping_add(2_000_000);
    let lh_sm = theme.font_size_sm * 1.2;
    let lh_lg = theme.font_size_lg * 1.2;

    let title_y = y + theme.padding_y;
    let title_id = core.text_pipeline.set_text(
        Some(key),
        title,
        glyphon::Metrics::new(theme.font_size_sm, lh_sm),
        Some(text_max_w),
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: title_id,
        x: x + theme.padding_x,
        y: title_y,
        clip: [x, y, w, h],
        color: color_to_f32(theme.base_content),
        z: 0.0,
    });

    let value_y = title_y + lh_sm + 4.0;
    let value_id = core.text_pipeline.set_text(
        Some(key.wrapping_add(1)),
        value,
        glyphon::Metrics::new(theme.font_size_lg, lh_lg),
        Some(text_max_w),
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: value_id,
        x: x + theme.padding_x,
        y: value_y,
        clip: [x, y, w, h],
        color: color_to_f32(theme.base_content),
        z: 0.0,
    });

    if let Some(desc) = description {
        let desc_y = value_y + lh_lg + 4.0;
        let desc_id = core.text_pipeline.set_text(
            Some(key.wrapping_add(2)),
            desc,
            glyphon::Metrics::new(theme.font_size_sm, lh_sm),
            Some(text_max_w),
            None,
        );
        core.draw_list.push_text(TextCall {
            buffer_id: desc_id,
            x: x + theme.padding_x,
            y: desc_y,
            clip: [x, y, w, h],
            color: color_to_f32(theme.secondary_content),
            z: 0.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> akar_layout::NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(200.0),
                height: akar_layout::length(120.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(200.0), Some(120.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn zero_area_does_nothing() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        stat(
            &mut core,
            &layout,
            node_id,
            "Title",
            "42",
            Some("Description"),
            &crate::AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn stat_with_description() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        stat(
            &mut core,
            &layout,
            node,
            "Revenue",
            "$12,345",
            Some("vs last month"),
            &crate::AKAR_THEME_DARK,
        );

        let total = core.draw_list.len();
        let quad_count = core.draw_list.sorted_quads().len();
        assert!(quad_count >= 1);
        assert!(total - quad_count >= 3);
    }

    #[test]
    fn stat_without_description() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        stat(
            &mut core,
            &layout,
            node,
            "Revenue",
            "$12,345",
            None,
            &crate::AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 3);
    }
}
