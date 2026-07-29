use akar_core::{AkarCore, QuadCall, TextCall, Z_TEXT_FOREGROUND};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_style::{
    resolve_text_style, resolved_to_attrs, resolved_to_metrics, FontFamily, FontWeight,
    ResolvedTextStyle, TextStyle,
};
use crate::AkarTheme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

fn link_defaults(theme: &AkarTheme) -> ResolvedTextStyle {
    ResolvedTextStyle {
        font_size: theme.font_size_base,
        line_height: theme.font_size_base * 1.2,
        color: theme.primary,
        font_weight: FontWeight::Normal,
        font_family: FontFamily::SansSerif,
        align: crate::text_style::TextAlign::Start,
        wrap: false,
    }
}

pub fn link(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    overrides: Option<TextStyle>,
    theme: &AkarTheme,
) -> LinkResult {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return LinkResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    }

    let hovered = core.input.is_hovering(rect);
    let pressed = core.input.is_pressed(rect);
    let clicked = core.input.is_clicked(rect);

    let defaults = link_defaults(theme);
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

    let text_width = {
        let measured = core.text_pipeline.measure(buffer_id, Some(rect[2]));
        measured.x
    };

    if hovered {
        let underline_height = 2.0;
        let underline_y = rect[1] + metrics.line_height - underline_height;
        core.draw_list.push_quad(QuadCall {
            rect: [rect[0], underline_y, text_width, underline_height],
            fill: color_to_f32(resolved.color),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: Z_TEXT_FOREGROUND,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0],
        y: rect[1],
        clip: rect,
        color: color_to_f32(resolved.color),
        z: 0.0,
    });

    LinkResult {
        clicked,
        hovered,
        pressed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> akar_layout::NodeId {
        let parent = layout.new_leaf(akar_layout::Style {
            padding: akar_layout::Rect {
                left: akar_layout::length(100.0_f32),
                top: akar_layout::length(100.0_f32),
                right: akar_layout::length(0.0_f32),
                bottom: akar_layout::length(0.0_f32),
            },
            ..Default::default()
        });
        let node = layout.new_leaf(akar_layout::Style {
            size: akar_layout::Size {
                width: akar_layout::length(400.0_f32),
                height: akar_layout::length(60.0_f32),
            },
            ..Default::default()
        });
        layout.add_child(parent, node);
        layout.compute(
            parent,
            (Some(800.0_f32), Some(400.0_f32)),
            |_, _, _, _, _| akar_layout::Size::ZERO,
        );
        node
    }

    #[test]
    fn zero_area_returns_all_false() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        let result = link(
            &mut core,
            &layout,
            node_id,
            "Click here",
            None,
            &AKAR_THEME_DARK,
        );

        assert!(!result.clicked);
        assert!(!result.hovered);
        assert!(!result.pressed);
    }

    #[test]
    fn link_renders_text() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        link(
            &mut core,
            &layout,
            node,
            "Click here",
            None,
            &AKAR_THEME_DARK,
        );

        let text_count = core.draw_list.len() - core.draw_list.sorted_quads().len();
        assert!(text_count >= 1);
    }

    #[test]
    fn link_returns_hit_state() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        let result = link(
            &mut core,
            &layout,
            node,
            "Click here",
            None,
            &AKAR_THEME_DARK,
        );

        assert!(!result.clicked);
        assert!(!result.hovered);
        assert!(!result.pressed);
    }

    #[test]
    fn link_with_style_override() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        let override_style = TextStyle {
            color: Some(0xff0000ff),
            ..TextStyle::empty()
        };

        link(
            &mut core,
            &layout,
            node,
            "Red link",
            Some(override_style),
            &AKAR_THEME_DARK,
        );

        let text_count = core.draw_list.len() - core.draw_list.sorted_quads().len();
        assert!(text_count >= 1);
    }

    #[test]
    fn link_no_underline_when_not_hovered() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        link(&mut core, &layout, node, "Hover me", None, &AKAR_THEME_DARK);

        assert_eq!(core.draw_list.sorted_quads().len(), 0);
    }

    #[test]
    fn link_default_uses_primary_color() {
        let defaults = link_defaults(&AKAR_THEME_DARK);
        assert_eq!(defaults.color, AKAR_THEME_DARK.primary);
        assert_eq!(defaults.font_weight, FontWeight::Normal);
        assert!(!defaults.wrap);
    }
}
