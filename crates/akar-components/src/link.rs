use akar_core::{AkarCore, QuadCall, TextCall, Z_TEXT_FOREGROUND};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_style::{
    resolve_align, resolve_text_style, resolved_to_font_request, resolved_to_metrics, FontFamily,
    FontWeight, ResolvedTextStyle, TextStyle,
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

    // The underline is a separate quad, not part of the shaped buffer, so it
    // must independently locate where cosmic-text placed the aligned text
    // within the full-width buffer. `align` here is the same physical
    // (Left/Right/Center) value handed to cosmic-text above, so this offset
    // matches its placement without re-deriving direction.
    let text_width = {
        let measured = core.text_pipeline.measure(buffer_id, Some(rect[2]));
        measured.x
    };
    let underline_offset = match align {
        glyphon::cosmic_text::Align::Left => 0.0,
        glyphon::cosmic_text::Align::Center => (rect[2] - text_width) * 0.5,
        glyphon::cosmic_text::Align::Right
        | glyphon::cosmic_text::Align::End
        | glyphon::cosmic_text::Align::Justified => rect[2] - text_width,
    };
    let text_x = rect[0] + underline_offset.max(0.0);

    if hovered {
        let underline_height = 2.0;
        let underline_y = rect[1] + metrics.line_height - underline_height;
        core.draw_list.push_quad(QuadCall {
            rect: [text_x, underline_y, text_width, underline_height],
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
    use crate::text_style::TextAlign;
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

    /// Epic 023 Task 9: cosmic-text now owns Start/End alignment inside the
    /// full-width buffer, so link's `TextCall.x` must always sit at
    /// `rect[0]` regardless of alignment or direction (the underline quad,
    /// which is not part of the shaped buffer, is allowed to shift with
    /// alignment on its own — this test targets only the text draw call).
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

                link(
                    &mut core,
                    &layout,
                    node,
                    "Click here",
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
                    .expect("link pushes exactly one text call");

                assert_eq!(
                    text_call.x, rect[0],
                    "align {align:?} direction {direction:?}: TextCall.x must equal rect[0]"
                );
            }
        }
    }
}
