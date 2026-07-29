use akar_core::{AkarCore, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_style::{
    resolve_text_style, resolved_to_attrs, resolved_to_metrics, FontFamily, FontWeight,
    ResolvedTextStyle, TextStyle,
};
use crate::AkarTheme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum HeadingLevel {
    #[default]
    H1,
    H2,
    H3,
    H4,
}

fn level_defaults(level: HeadingLevel, theme: &AkarTheme) -> ResolvedTextStyle {
    let (font_size, font_weight) = match level {
        HeadingLevel::H1 => (theme.font_size_heading_1, FontWeight::Bold),
        HeadingLevel::H2 => (theme.font_size_heading_2, FontWeight::Bold),
        HeadingLevel::H3 => (theme.font_size_heading_3, FontWeight::Bold),
        HeadingLevel::H4 => (theme.font_size_heading_4, FontWeight::Semibold),
    };

    ResolvedTextStyle {
        font_size,
        line_height: font_size * 1.2,
        color: theme.base_content,
        font_weight,
        font_family: FontFamily::SansSerif,
        align: crate::text_style::TextAlign::Start,
        wrap: false,
    }
}

pub fn heading(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    level: HeadingLevel,
    overrides: Option<TextStyle>,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let defaults = level_defaults(level, theme);
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
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> akar_layout::NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(400.0_f32),
                height: akar_layout::length(60.0_f32),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(400.0_f32), Some(60.0_f32)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn zero_area_does_not_push_text() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        heading(
            &mut core,
            &layout,
            node_id,
            "Hello",
            HeadingLevel::H1,
            None,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn heading_h1_renders_text() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        heading(
            &mut core,
            &layout,
            node,
            "Heading 1",
            HeadingLevel::H1,
            None,
            &AKAR_THEME_DARK,
        );

        let text_count = core.draw_list.len() - core.draw_list.sorted_quads().len();
        assert!(text_count >= 1);
    }

    #[test]
    fn heading_all_levels_render() {
        let levels = [
            HeadingLevel::H1,
            HeadingLevel::H2,
            HeadingLevel::H3,
            HeadingLevel::H4,
        ];

        for level in &levels {
            let mut layout = akar_layout::Layout::new();
            let node = sized_node(&mut layout);
            let mut core = AkarCore::mock();

            heading(
                &mut core,
                &layout,
                node,
                "Title",
                *level,
                None,
                &AKAR_THEME_DARK,
            );

            let text_count = core.draw_list.len() - core.draw_list.sorted_quads().len();
            assert!(text_count >= 1, "level {:?} did not produce text", level);
        }
    }

    #[test]
    fn heading_with_style_override() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        let override_style = TextStyle {
            color: Some(0xff0000ff),
            ..TextStyle::empty()
        };

        heading(
            &mut core,
            &layout,
            node,
            "Red Heading",
            HeadingLevel::H1,
            Some(override_style),
            &AKAR_THEME_DARK,
        );

        let text_count = core.draw_list.len() - core.draw_list.sorted_quads().len();
        assert!(text_count >= 1);
    }
}
