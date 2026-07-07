use akar_core::{AkarCore, QuadCall, Z_OVERLAY};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub struct DropdownState {
    pub is_open: bool,
    pub content_rect: [f32; 4],
}

pub fn dropdown_begin(
    core: &mut AkarCore,
    anchor_rect: [f32; 4],
    item_height: f32,
    viewport_rect: [f32; 4],
    is_open: bool,
    theme: &AkarTheme,
) -> DropdownState {
    if !is_open {
        return DropdownState {
            is_open: false,
            content_rect: [0.0; 4],
        };
    }

    let total_items_visible = 4.0;
    let width = anchor_rect[2];
    let total_height = item_height * total_items_visible;

    let mut y = anchor_rect[1] + anchor_rect[3];
    if y + total_height > viewport_rect[1] + viewport_rect[3] {
        y = anchor_rect[1] - total_height;
    }

    let x = anchor_rect[0];
    let dropdown_rect = [x, y, width, total_height];

    core.draw_list.push_quad(QuadCall {
        rect: dropdown_rect,
        fill: color_to_f32(theme.base_200),
        border_color: color_to_f32(theme.base_300),
        corner_radii: [theme.radius_field; 4],
        border_width: 1.0,
        z: Z_OVERLAY,
        shadow_blur: 12.0,
        shadow_spread: 0.0,
        shadow_color: [0.0, 0.0, 0.0, 0.25],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let inset = 1.0;
    let content_rect = [
        x + inset,
        y + inset,
        width - inset * 2.0,
        total_height - inset * 2.0,
    ];

    core.draw_list.push_scissor(content_rect);

    DropdownState {
        is_open: true,
        content_rect,
    }
}

pub fn dropdown_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;

    fn make_core() -> AkarCore {
        AkarCore::mock()
    }

    #[test]
    fn closed_dropdown_renders_nothing() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let state = dropdown_begin(
            &mut core,
            [100.0, 200.0, 150.0, 40.0],
            28.0,
            [0.0, 0.0, 400.0, 600.0],
            false,
            &AKAR_THEME_DARK,
        );

        assert!(!state.is_open);
        assert_eq!(core.draw_list.len(), 0);
        dropdown_end(&mut core);
    }

    #[test]
    fn open_dropdown_renders_background() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let state = dropdown_begin(
            &mut core,
            [100.0, 200.0, 150.0, 40.0],
            28.0,
            [0.0, 0.0, 400.0, 600.0],
            true,
            &AKAR_THEME_DARK,
        );

        assert!(state.is_open);

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].z, Z_OVERLAY);
        assert!(core.draw_list.active_scissor().is_some());

        dropdown_end(&mut core);
    }

    #[test]
    fn opens_below_anchor_by_default() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let anchor_rect = [100.0, 200.0, 150.0, 40.0];
        dropdown_begin(
            &mut core,
            anchor_rect,
            28.0,
            [0.0, 0.0, 400.0, 600.0],
            true,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads[0].rect[1], anchor_rect[1] + anchor_rect[3]);

        dropdown_end(&mut core);
    }

    #[test]
    fn opens_above_anchor_when_near_bottom() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let anchor_rect = [100.0, 550.0, 150.0, 40.0];
        let total_height = 28.0 * 4.0;
        dropdown_begin(
            &mut core,
            anchor_rect,
            28.0,
            [0.0, 0.0, 400.0, 600.0],
            true,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads[0].rect[1], anchor_rect[1] - total_height);

        dropdown_end(&mut core);
    }

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        dropdown_begin(
            &mut core,
            [100.0, 200.0, 150.0, 40.0],
            28.0,
            [0.0, 0.0, 400.0, 600.0],
            true,
            &AKAR_THEME_DARK,
        );

        assert!(core.draw_list.active_scissor().is_some());

        dropdown_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }
}
