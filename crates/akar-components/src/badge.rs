use crate::color::color_to_f32;
use crate::label::label;
use crate::text_style::TextStyle;
use crate::AkarTheme;
use akar_core::{AkarCore, QuadCall};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::{length, Layout, Size, Style};

    fn node_100x30(layout: &mut Layout) -> NodeId {
        let n = layout.new_leaf(Style {
            size: Size {
                width: length(100.0),
                height: length(30.0),
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
}
