use akar_core::{AkarCore, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_style::{
    resolve_align, resolve_text_style, resolved_to_font_request, resolved_to_metrics, FontFamily,
    FontWeight, ResolvedTextStyle, TextStyle,
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
    let font = resolved_to_font_request(&resolved);

    let align = resolve_align(resolved.align, layout.direction());
    let buffer_id = core.text_pipeline.set_text_styled(
        Some(layout.widget_id(node_id)),
        text,
        metrics,
        Some(rect[2]),
        None,
        font,
        Some(align),
    );

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0],
        y: rect[1],
        clip: rect,
        color: color_to_f32(resolved.color),
        z: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_style::TextAlign;
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

    /// Epic 023 Task 9: cosmic-text now owns Start/End alignment inside the
    /// full-width buffer, so the paragraph's `TextCall.x` must always sit at
    /// `rect[0]` regardless of alignment or direction — never shifted by a
    /// manual x-offset, which would double-apply cosmic-text's own
    /// alignment for RTL content.
    #[test]
    fn text_call_x_is_rect_origin_across_align_and_direction() {
        use akar_core::DrawCall;
        use akar_layout::AkarDirection;

        for direction in [AkarDirection::Ltr, AkarDirection::Rtl] {
            for align in [TextAlign::Start, TextAlign::Center, TextAlign::End] {
                let mut layout = akar_layout::Layout::new();
                layout.set_direction(direction);
                let node = sized_node(&mut layout);
                let mut core = AkarCore::mock();

                let override_style = TextStyle {
                    align: Some(align),
                    ..TextStyle::empty()
                };

                paragraph(
                    &mut core,
                    &layout,
                    node,
                    "Hello world",
                    Some(override_style),
                    &AKAR_THEME_DARK,
                );

                let rect = layout.rect(node);
                let text_call = core
                    .draw_list
                    .text_calls()
                    .iter()
                    .find_map(|call| match call {
                        DrawCall::Text(text) => Some(text),
                        DrawCall::Quad(_) => None,
                    })
                    .expect("paragraph pushes exactly one text call");

                assert_eq!(
                    text_call.x, rect[0],
                    "align {align:?} direction {direction:?}: TextCall.x must equal rect[0]"
                );
            }
        }
    }
}
