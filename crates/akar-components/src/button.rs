use akar_core::AkarCore;
use akar_core::{QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_style::TextStyle;
use crate::AkarTheme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Solid,
    Outline,
    Ghost,
}

pub struct ButtonResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

pub struct ButtonStyle {
    pub fill: Option<u32>,
    pub hover_fill: Option<u32>,
    pub pressed_fill: Option<u32>,
    pub border_color: Option<u32>,
    pub content_color: Option<u32>,
    pub text_style: Option<TextStyle>,
}

impl ButtonStyle {
    pub fn empty() -> Self {
        Self {
            fill: None,
            hover_fill: None,
            pressed_fill: None,
            border_color: None,
            content_color: None,
            text_style: None,
        }
    }
}

fn scale_color(c: u32, factor: f32) -> u32 {
    let r = (((c >> 24) & 0xFF) as f32 * factor).min(255.0) as u32;
    let g = (((c >> 16) & 0xFF) as f32 * factor).min(255.0) as u32;
    let b = (((c >> 8) & 0xFF) as f32 * factor).min(255.0) as u32;
    let a = c & 0xFF;
    (r << 24) | (g << 16) | (b << 8) | a
}

pub fn button(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    label: &str,
    variant: ButtonVariant,
    theme: &AkarTheme,
) -> ButtonResult {
    let style = ButtonStyle::empty();
    button_styled(core, layout, node_id, label, variant, &style, theme)
}

pub fn button_styled(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    label: &str,
    variant: ButtonVariant,
    style: &ButtonStyle,
    theme: &AkarTheme,
) -> ButtonResult {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return ButtonResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    }

    let hovered = core.input.is_hovering(rect);
    let pressed = core.input.is_pressed(rect);
    let clicked = core.input.is_clicked(rect);

    let (fill_color, border_color) = match variant {
        ButtonVariant::Solid => {
            let base = if pressed {
                style
                    .pressed_fill
                    .unwrap_or_else(|| scale_color(theme.primary, 0.8))
            } else if hovered {
                style
                    .hover_fill
                    .unwrap_or_else(|| scale_color(theme.primary, 1.1))
            } else {
                style.fill.unwrap_or(theme.primary)
            };
            (base, style.border_color.unwrap_or(theme.primary))
        }
        ButtonVariant::Outline => {
            let border = if hovered {
                style
                    .hover_fill
                    .unwrap_or_else(|| scale_color(theme.primary, 1.1))
            } else {
                style.border_color.unwrap_or(theme.primary)
            };
            (style.fill.unwrap_or(0x00000000u32), border)
        }
        ButtonVariant::Ghost => {
            let fill = if hovered {
                style
                    .hover_fill
                    .unwrap_or_else(|| scale_color(theme.primary, 1.1))
            } else {
                style.fill.unwrap_or(0x00000000u32)
            };
            (fill, style.border_color.unwrap_or(0x00000000u32))
        }
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(fill_color),
        border_color: color_to_f32(border_color),
        corner_radii: [theme.radius_field; 4],
        border_width: theme.border_width,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let buffer_id = core.text_pipeline.set_text(
        Some(layout.widget_id(node_id)),
        label,
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
        Some(rect[2]),
        None,
        None,
    );

    let text_color = style.content_color.unwrap_or_else(|| match variant {
        ButtonVariant::Solid => theme.primary_content,
        _ => theme.base_content,
    });

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0] + theme.border_width + theme.padding_x,
        y: rect[1] + theme.border_width + theme.padding_y,
        clip: rect,
        color: color_to_f32(text_color),
        z: 0.0,
    });

    ButtonResult {
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

    #[test]
    fn zero_area_returns_all_false() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();

        let result = button(
            &mut core,
            &layout,
            node_id,
            "Click",
            ButtonVariant::Solid,
            &AKAR_THEME_DARK,
        );

        assert!(!result.clicked);
        assert!(!result.hovered);
        assert!(!result.pressed);
    }

    #[test]
    fn styled_uses_custom_fill() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(100.0),
                height: akar_layout::length(40.0),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(200.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        core.input.set_mouse_pos(500.0, 500.0);
        core.input.begin_frame();
        core.draw_list.begin_frame(1.0);

        let style = ButtonStyle {
            fill: Some(0xFF0000FF),
            ..ButtonStyle::empty()
        };

        button_styled(
            &mut core,
            &layout,
            node_id,
            "Click",
            ButtonVariant::Solid,
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
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        let style = ButtonStyle {
            fill: Some(0xFF0000FF),
            ..ButtonStyle::empty()
        };

        let result = button_styled(
            &mut core,
            &layout,
            node_id,
            "Click",
            ButtonVariant::Solid,
            &style,
            &AKAR_THEME_DARK,
        );

        assert!(!result.clicked);
        assert!(!result.hovered);
        assert!(!result.pressed);
    }
}
