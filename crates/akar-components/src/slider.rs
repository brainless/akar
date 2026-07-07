use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub fn slider(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: &mut f32,
    min: f32,
    max: f32,
    theme: &AkarTheme,
) -> bool {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return false;
    }

    let track_h = 6.0;
    let track_x = rect[0];
    let track_y = rect[1] + (rect[3] - track_h) * 0.5;
    let track_w = rect[2];

    let range = max - min;
    let fraction = if range != 0.0 {
        ((*value - min) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fill_w = track_w * fraction;

    core.draw_list.push_quad(QuadCall {
        rect: [track_x, track_y, track_w, track_h],
        fill: color_to_f32(theme.base_300),
        border_color: [0.0; 4],
        corner_radii: [track_h * 0.5; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    if fill_w > 0.0 {
        core.draw_list.push_quad(QuadCall {
            rect: [track_x, track_y, fill_w, track_h],
            fill: color_to_f32(theme.primary),
            border_color: [0.0; 4],
            corner_radii: [track_h * 0.5; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    let thumb_size = 14.0;
    let thumb_x = track_x + fill_w - thumb_size * 0.5;
    let thumb_y = rect[1] + (rect[3] - thumb_size) * 0.5;

    core.draw_list.push_quad(QuadCall {
        rect: [thumb_x, thumb_y, thumb_size, thumb_size],
        fill: [1.0; 4],
        border_color: color_to_f32(theme.base_300),
        corner_radii: [thumb_size * 0.5; 4],
        border_width: 1.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    if core.input.is_pressed(rect) {
        let mouse_x = core.input.mouse_pos.x;
        let normalized = ((mouse_x - track_x) / track_w).clamp(0.0, 1.0);
        if range != 0.0 {
            *value = min + normalized * range;
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_returns_false() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        let mut value = 0.5;

        let result = slider(
            &mut core,
            &layout,
            node_id,
            &mut value,
            0.0,
            1.0,
            &AKAR_THEME_DARK,
        );

        assert!(!result);
    }
}
