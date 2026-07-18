use akar_core::AkarCore;
use akar_core::{QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
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
                scale_color(theme.primary, 0.8)
            } else if hovered {
                scale_color(theme.primary, 1.1)
            } else {
                theme.primary
            };
            (base, theme.primary)
        }
        ButtonVariant::Outline => {
            let border = if hovered {
                scale_color(theme.primary, 1.1)
            } else {
                theme.primary
            };
            (0x00000000u32, border)
        }
        ButtonVariant::Ghost => {
            let fill = if hovered {
                scale_color(theme.primary, 1.1)
            } else {
                0x00000000u32
            };
            (fill, 0x00000000u32)
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
    );

    let text_color = match variant {
        ButtonVariant::Solid => color_to_f32(theme.primary_content),
        _ => color_to_f32(theme.base_content),
    };

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0] + theme.border_width + theme.padding_x,
        y: rect[1] + theme.border_width + theme.padding_y,
        clip: rect,
        color: text_color,
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
}
