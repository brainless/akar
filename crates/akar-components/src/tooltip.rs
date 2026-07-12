use akar_core::{AkarCore, QuadCall, TextCall, Z_OVERLAY};

use crate::color::color_to_f32;
use crate::AkarTheme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TooltipSide {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct TooltipResponse {
    pub visible: bool,
}

pub fn position_tooltip(
    trigger_rect: [f32; 4],
    tooltip_size: [f32; 2],
    viewport_rect: [f32; 4],
    preferred_side: TooltipSide,
) -> [f32; 4] {
    let [tw, th] = tooltip_size;
    let [tx, ty, tw_tr, th_tr] = trigger_rect;
    let [vx, vy, vw, vh] = viewport_rect;

    let (mut x, mut y) = match preferred_side {
        TooltipSide::Top => {
            let x = tx + tw_tr / 2.0 - tw / 2.0;
            let y = ty - th;
            (x, y)
        }
        TooltipSide::Bottom => {
            let x = tx + tw_tr / 2.0 - tw / 2.0;
            let y = ty + th_tr;
            (x, y)
        }
        TooltipSide::Left => {
            let x = tx - tw;
            let y = ty + th_tr / 2.0 - th / 2.0;
            (x, y)
        }
        TooltipSide::Right => {
            let x = tx + tw_tr;
            let y = ty + th_tr / 2.0 - th / 2.0;
            (x, y)
        }
    };

    match preferred_side {
        TooltipSide::Top => {
            if y < vy {
                y = ty + th_tr;
            }
        }
        TooltipSide::Bottom => {
            if y + th > vy + vh {
                y = ty - th;
            }
        }
        TooltipSide::Left => {
            if x < vx {
                x = tx + tw_tr;
            }
        }
        TooltipSide::Right => {
            if x + tw > vx + vw {
                x = tx - tw;
            }
        }
    }

    x = x.max(vx).min(vx + vw - tw);
    y = y.max(vy).min(vy + vh - th);

    [x, y, tw, th]
}

pub fn tooltip(
    core: &mut AkarCore,
    trigger_rect: [f32; 4],
    text: &str,
    preferred_side: TooltipSide,
    theme: &AkarTheme,
    viewport_rect: [f32; 4],
) -> TooltipResponse {
    if !core.input.is_hovering(trigger_rect) {
        return TooltipResponse { visible: false };
    }

    let metrics = glyphon::Metrics::new(theme.font_size_sm, theme.font_size_sm * 1.2);

    let buffer_id = core.text_pipeline.set_text(None, text, metrics, None, None);

    let text_size = core.text_pipeline.measure(buffer_id, None);

    let padding = 4.0;
    let tooltip_w = text_size.x + padding * 2.0;
    let tooltip_h = text_size.y + padding * 2.0;

    let tooltip_rect = position_tooltip(
        trigger_rect,
        [tooltip_w, tooltip_h],
        viewport_rect,
        preferred_side,
    );

    core.text_pipeline.set_text(
        Some(buffer_id),
        text,
        metrics,
        Some(tooltip_w - padding * 2.0),
        None,
    );

    let inner_x = tooltip_rect[0] + padding;
    let inner_y = tooltip_rect[1] + padding;
    let inner_w = tooltip_rect[2] - padding * 2.0;
    let inner_h = tooltip_rect[3] - padding * 2.0;

    core.draw_list.push_quad(QuadCall {
        rect: tooltip_rect,
        fill: color_to_f32(theme.neutral),
        border_color: [0.0; 4],
        corner_radii: [theme.radius_field; 4],
        border_width: 0.0,
        z: Z_OVERLAY,
        shadow_blur: 8.0,
        shadow_spread: 0.0,
        shadow_color: [0.0, 0.0, 0.0, 0.3],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: inner_x,
        y: inner_y,
        clip: [inner_x, inner_y, inner_w, inner_h],
        color: color_to_f32(theme.neutral_content),
        z: Z_OVERLAY,
    });

    TooltipResponse { visible: true }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;

    fn make_core() -> AkarCore {
        AkarCore::mock()
    }

    #[test]
    fn tooltip_not_visible_when_not_hovered() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        core.input.set_mouse_pos(0.0, 0.0);

        let resp = tooltip(
            &mut core,
            [100.0, 100.0, 50.0, 30.0],
            "Hello",
            TooltipSide::Top,
            &AKAR_THEME_DARK,
            [0.0, 0.0, 400.0, 600.0],
        );

        assert!(!resp.visible);
        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn tooltip_visible_when_hovered() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        core.input.set_mouse_pos(125.0, 115.0);

        let resp = tooltip(
            &mut core,
            [100.0, 100.0, 50.0, 30.0],
            "Hello",
            TooltipSide::Top,
            &AKAR_THEME_DARK,
            [0.0, 0.0, 400.0, 600.0],
        );

        assert!(resp.visible);
        assert!(!core.draw_list.is_empty());
    }

    #[test]
    fn position_tooltip_above() {
        let result = position_tooltip(
            [100.0, 200.0, 100.0, 50.0],
            [120.0, 40.0],
            [0.0, 0.0, 400.0, 600.0],
            TooltipSide::Top,
        );

        assert_eq!(result, [90.0, 160.0, 120.0, 40.0]);
    }

    #[test]
    fn position_tooltip_flips_to_bottom() {
        let result = position_tooltip(
            [100.0, 10.0, 100.0, 50.0],
            [120.0, 40.0],
            [0.0, 0.0, 400.0, 600.0],
            TooltipSide::Top,
        );

        assert_eq!(result, [90.0, 60.0, 120.0, 40.0]);
    }

    #[test]
    fn position_tooltip_clamps_to_viewport() {
        let result = position_tooltip(
            [350.0, 200.0, 100.0, 50.0],
            [120.0, 40.0],
            [0.0, 0.0, 400.0, 600.0],
            TooltipSide::Top,
        );

        assert_eq!(result[0], 280.0);
        assert_eq!(result[1], 160.0);
        assert_eq!(result[2], 120.0);
        assert_eq!(result[3], 40.0);
    }
}
