use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::color::{color_to_f32, scale_color};
use crate::AkarTheme;

pub fn switch(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    on: &mut bool,
    theme: &AkarTheme,
) -> bool {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return false;
    }

    let hovered = core.input.is_hovering(rect);
    let clicked = core.input.is_clicked(rect);

    if clicked {
        *on = !*on;
    }

    let track_w = 36.0;
    let track_h = 20.0;
    let track_x = rect[0];
    let track_y = rect[1] + (rect[3] - track_h) * 0.5;
    let track_rect = [track_x, track_y, track_w, track_h];

    let track_color = if *on {
        if hovered {
            scale_color(theme.primary, 1.2)
        } else {
            theme.primary
        }
    } else {
        if hovered {
            scale_color(theme.base_300, 1.2)
        } else {
            theme.base_300
        }
    };

    core.draw_list.push_quad(QuadCall {
        rect: track_rect,
        fill: color_to_f32(track_color),
        border_color: [0.0; 4],
        corner_radii: [10.0; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let thumb_size = 16.0;
    let thumb_y = track_y + (track_h - thumb_size) * 0.5;
    let thumb_x = if *on {
        track_x + track_w - 2.0 - thumb_size
    } else {
        track_x + 2.0
    };

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

    clicked
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
        let mut on = false;

        let result = switch(&mut core, &layout, node_id, &mut on, &AKAR_THEME_DARK);

        assert!(!result);
        assert!(!on);
    }
}
