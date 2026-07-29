use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub struct SeparatorStyle {
    pub color: Option<u32>,
    pub thickness: Option<f32>,
}

impl SeparatorStyle {
    pub fn empty() -> Self {
        Self {
            color: None,
            thickness: None,
        }
    }
}

pub fn separator(core: &mut AkarCore, layout: &Layout, node_id: NodeId, theme: &AkarTheme) {
    let style = SeparatorStyle::empty();
    separator_styled(core, layout, node_id, &style, theme)
}

pub fn separator_styled(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    style: &SeparatorStyle,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let color = style.color.unwrap_or(theme.base_300);

    let draw_rect = if let Some(thickness) = style.thickness {
        [
            rect[0],
            rect[1] + (rect[3] - thickness) * 0.5,
            rect[2],
            thickness,
        ]
    } else {
        rect
    };

    core.draw_list.push_quad(QuadCall {
        rect: draw_rect,
        fill: color_to_f32(color),
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_pushes_no_quad() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        separator(&mut core, &layout, node_id, &AKAR_THEME_DARK);

        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn styled_uses_custom_color() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(200.0),
                height: akar_layout::length(10.0),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(200.0), Some(10.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let style = SeparatorStyle {
            color: Some(0xFF0000FF),
            ..SeparatorStyle::empty()
        };

        separator_styled(&mut core, &layout, node_id, &style, &AKAR_THEME_DARK);

        let quads = core.draw_list.sorted_quads();
        assert!(!quads.is_empty());
        assert_eq!(quads[0].fill, color_to_f32(0xFF0000FF));
    }

    #[test]
    fn styled_preserves_zero_area() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        let style = SeparatorStyle {
            color: Some(0xFF0000FF),
            ..SeparatorStyle::empty()
        };

        separator_styled(&mut core, &layout, node_id, &style, &AKAR_THEME_DARK);

        assert!(core.draw_list.sorted_quads().is_empty());
    }
}
