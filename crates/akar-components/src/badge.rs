use crate::color::color_to_f32;
use crate::label::label;
use crate::text_style::TextStyle;
use crate::AkarTheme;
use akar_core::{AkarCore, QuadCall, TextCall, Z_BASE, Z_TEXT_FOREGROUND};
use akar_layout::{Layout, NodeId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeVariant {
    Default,
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

pub struct BadgeStyle {
    pub fill: Option<u32>,
    pub border_color: Option<u32>,
    pub content_color: Option<u32>,
    pub text_style: Option<TextStyle>,
}

impl BadgeStyle {
    pub fn empty() -> Self {
        Self {
            fill: None,
            border_color: None,
            content_color: None,
            text_style: None,
        }
    }
}

pub fn badge(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    variant: BadgeVariant,
    theme: &AkarTheme,
) {
    let style = BadgeStyle::empty();
    badge_styled(core, layout, node_id, text, variant, &style, theme)
}

pub fn badge_styled(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    variant: BadgeVariant,
    style: &BadgeStyle,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let (bg, fg) = match variant {
        BadgeVariant::Default => (theme.base_300, theme.base_content),
        BadgeVariant::Primary => (theme.primary, theme.primary_content),
        BadgeVariant::Success => (theme.success, theme.success_content),
        BadgeVariant::Warning => (theme.warning, theme.warning_content),
        BadgeVariant::Error => (theme.error, theme.error_content),
        BadgeVariant::Info => (theme.info, theme.info_content),
    };

    let fill_color = style.fill.unwrap_or(bg);
    let content_color = style.content_color.unwrap_or(fg);

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(fill_color),
        border_color: color_to_f32(style.border_color.unwrap_or(0x00000000)),
        corner_radii: [theme.radius_field; 4],
        border_width: if style.border_color.is_some() {
            theme.border_width
        } else {
            0.0
        },
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    label(core, layout, node_id, text, content_color, theme);
}

/// Draws a content-sized pill inside a caller-provided rectangle. The text ID
/// must stay stable for the same item across frames; display-only buffers are
/// released after they leave the viewport.
pub fn badge_pill(
    core: &mut AkarCore,
    rect: [f32; 4],
    text_id: u64,
    text: &str,
    fill: u32,
    content: u32,
    theme: &AkarTheme,
) {
    if rect[2] <= 0.0 || rect[3] <= 0.0 || text.is_empty() {
        return;
    }

    let inset = 4.0_f32.min(rect[2] * 0.5).min(rect[3] * 0.5);
    let height = rect[3] - inset * 2.0;
    let max_width = rect[2] - inset * 2.0;
    if height <= 0.0 || max_width <= 0.0 {
        return;
    }

    let font_size = theme.font_size_sm.min(height * 0.65);
    let metrics = glyphon::Metrics::new(font_size, font_size * 1.2);
    let buffer_id = core
        .text_pipeline
        .set_text_transient(text_id, text, metrics, None, None, None);
    let text_width = core.text_pipeline.measure(buffer_id, None).x;
    let padding_x = 10.0_f32.min(max_width * 0.25);
    let width = (text_width + padding_x * 2.0).min(max_width);
    let pill_rect = [rect[0] + inset, rect[1] + inset, width, height];

    core.draw_list.push_quad(QuadCall {
        rect: pill_rect,
        fill: color_to_f32(fill),
        border_color: [0.0; 4],
        corner_radii: [height * 0.5; 4],
        border_width: 0.0,
        z: Z_BASE + 0.1,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let clip = if let Some(scissor) = core.draw_list.active_scissor() {
        let scale = core.draw_list.scale_factor();
        let left = pill_rect[0].max(scissor[0] / scale);
        let top = pill_rect[1].max(scissor[1] / scale);
        let right = (pill_rect[0] + pill_rect[2]).min((scissor[0] + scissor[2]) / scale);
        let bottom = (pill_rect[1] + pill_rect[3]).min((scissor[1] + scissor[3]) / scale);
        [left, top, (right - left).max(0.0), (bottom - top).max(0.0)]
    } else {
        pill_rect
    };
    core.draw_list.push_text(TextCall {
        buffer_id,
        x: pill_rect[0] + padding_x,
        y: pill_rect[1] + (height - font_size) * 0.5,
        clip,
        color: color_to_f32(content),
        z: Z_TEXT_FOREGROUND,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::{length, Layout, Size, Style};

    fn node_100x30(layout: &mut Layout) -> NodeId {
        let n = layout.new_leaf(Style {
            size: Size {
                width: length(100.0_f32),
                height: length(30.0_f32),
            },
            ..Default::default()
        });
        layout.compute(n, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);
        n
    }

    #[test]
    fn styled_uses_custom_fill() {
        let mut layout = Layout::new();
        let node = node_100x30(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let style = BadgeStyle {
            fill: Some(0xFF0000FF),
            ..BadgeStyle::empty()
        };

        badge_styled(
            &mut core,
            &layout,
            node,
            "New",
            BadgeVariant::Primary,
            &style,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert!(!quads.is_empty());
        assert_eq!(quads[0].fill, color_to_f32(0xFF0000FF));
    }

    #[test]
    fn styled_preserves_zero_area() {
        let mut layout = Layout::new();
        let node = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        let style = BadgeStyle {
            fill: Some(0xFF0000FF),
            ..BadgeStyle::empty()
        };

        badge_styled(
            &mut core,
            &layout,
            node,
            "New",
            BadgeVariant::Primary,
            &style,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn pill_fits_and_rounds_inside_its_rect() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        badge_pill(
            &mut core,
            [20.0, 30.0, 180.0, 32.0],
            42,
            "Switzerland",
            0x1D4ED8FF,
            0xFFFFFFFF,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 1);
        let pill = quads[0];
        assert!(pill.rect[0] > 20.0 && pill.rect[0] + pill.rect[2] < 200.0);
        assert!(pill.rect[1] > 30.0 && pill.rect[1] + pill.rect[3] < 62.0);
        assert_eq!(pill.corner_radii, [pill.rect[3] * 0.5; 4]);
    }

    #[test]
    fn pill_text_stays_inside_active_scissor() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(2.0);
        core.draw_list.push_scissor([0.0, 36.0, 200.0, 100.0]);
        badge_pill(
            &mut core,
            [20.0, 26.0, 180.0, 32.0],
            42,
            "Switzerland",
            0x1D4ED8FF,
            0xFFFFFFFF,
            &AKAR_THEME_DARK,
        );

        let text = core
            .draw_list
            .text_calls()
            .iter()
            .find_map(|call| match call {
                akar_core::DrawCall::Text(text) => Some(text),
                _ => None,
            })
            .unwrap();
        assert_eq!(text.clip[1], 36.0);
        assert_eq!(text.clip[3], 18.0);
    }
}
