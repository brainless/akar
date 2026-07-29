use akar_core::{AkarCore, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_style::{
    resolve_text_style, resolved_to_attrs, resolved_to_metrics, FontFamily, FontWeight,
    ResolvedTextStyle, TextAlign, TextStyle,
};
use crate::AkarTheme;

fn paragraph_defaults(theme: &AkarTheme) -> ResolvedTextStyle {
    let font_size = theme.font_size_base;
    ResolvedTextStyle {
        font_size,
        line_height: font_size * 1.5,
        color: theme.base_content,
        font_weight: FontWeight::Normal,
        font_family: FontFamily::SansSerif,
        align: crate::text_style::TextAlign::Start,
        wrap: true,
    }
}

pub fn paragraph(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    overrides: Option<TextStyle>,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let defaults = paragraph_defaults(theme);
    let resolved = resolve_text_style(theme, &defaults, overrides.as_ref());
    let metrics = resolved_to_metrics(&resolved);
    let attrs = resolved_to_attrs(&resolved);

    let buffer_id = core.text_pipeline.set_text(
        Some(layout.widget_id(node_id)),
        text,
        metrics,
        Some(rect[2]),
        None,
        Some(attrs),
    );

    let text_width = core.text_pipeline.measure(buffer_id, Some(rect[2])).x;
    let x_offset = match resolved.align {
        TextAlign::Start => 0.0,
        TextAlign::Center => (rect[2] - text_width) * 0.5,
        TextAlign::End => rect[2] - text_width,
    };

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0] + x_offset.max(0.0),
        y: rect[1],
        clip: rect,
        color: color_to_f32(resolved.color),
        z: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> akar_layout::NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(300.0_f32),
                height: akar_layout::length(200.0_f32),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(300.0_f32), Some(200.0_f32)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn zero_area_does_not_push_text() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        paragraph(
            &mut core,
            &layout,
            node_id,
            "Hello world",
            None,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn paragraph_renders_text() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        paragraph(
            &mut core,
            &layout,
            node,
            "A paragraph of text.",
            None,
            &AKAR_THEME_DARK,
        );

        let total = core.draw_list.len();
        let quad_count = core.draw_list.sorted_quads().len();
        assert!(total - quad_count >= 1);
    }

    #[test]
    fn paragraph_with_long_text() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        let long_text = "This is a longer paragraph that should wrap across multiple lines when rendered with the text pipeline. It contains enough text to trigger wrapping behavior at the configured width.";

        paragraph(&mut core, &layout, node, long_text, None, &AKAR_THEME_DARK);

        let total = core.draw_list.len();
        let quad_count = core.draw_list.sorted_quads().len();
        assert!(total - quad_count >= 1);
    }

    #[test]
    fn paragraph_with_style_override() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        let override_style = TextStyle {
            color: Some(0x00ff00ff),
            font_size: Some(14.0),
            ..TextStyle::empty()
        };

        paragraph(
            &mut core,
            &layout,
            node,
            "Styled paragraph",
            Some(override_style),
            &AKAR_THEME_DARK,
        );

        let total = core.draw_list.len();
        let quad_count = core.draw_list.sorted_quads().len();
        assert!(total - quad_count >= 1);
    }
}
