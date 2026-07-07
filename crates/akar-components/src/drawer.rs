use akar_core::AkarCore;
use akar_core::QuadCall;
use akar_core::Z_FLOAT;
use akar_core::Z_SCRIM;

use crate::color::color_to_f32;
use crate::AkarTheme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawerEdge {
    Left,
    Right,
}

pub struct DrawerResponse {
    pub close_requested: bool,
}

pub fn drawer_begin(
    core: &mut AkarCore,
    viewport_rect: [f32; 4],
    edge: DrawerEdge,
    panel_width: f32,
    theme: &AkarTheme,
) -> DrawerResponse {
    if panel_width <= 0.0 {
        return DrawerResponse {
            close_requested: false,
        };
    }

    let panel_width = panel_width.min(viewport_rect[2]);

    let panel_rect = match edge {
        DrawerEdge::Left => [
            viewport_rect[0],
            viewport_rect[1],
            panel_width,
            viewport_rect[3],
        ],
        DrawerEdge::Right => [
            viewport_rect[0] + viewport_rect[2] - panel_width,
            viewport_rect[1],
            panel_width,
            viewport_rect[3],
        ],
    };

    let scrim_rect = match edge {
        DrawerEdge::Left => [
            viewport_rect[0] + panel_width,
            viewport_rect[1],
            viewport_rect[2] - panel_width,
            viewport_rect[3],
        ],
        DrawerEdge::Right => [
            viewport_rect[0],
            viewport_rect[1],
            viewport_rect[2] - panel_width,
            viewport_rect[3],
        ],
    };

    core.draw_list.push_quad(QuadCall {
        rect: scrim_rect,
        fill: color_to_f32(0x00000080),
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: Z_SCRIM,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let corner_radii = match edge {
        DrawerEdge::Left => [0.0, theme.radius_box, theme.radius_box, 0.0],
        DrawerEdge::Right => [theme.radius_box, 0.0, 0.0, theme.radius_box],
    };

    let shadow_offset = match edge {
        DrawerEdge::Left => [2.0, 0.0],
        DrawerEdge::Right => [-2.0, 0.0],
    };

    core.draw_list.push_quad(QuadCall {
        rect: panel_rect,
        fill: color_to_f32(theme.base_200),
        border_color: [0.0; 4],
        corner_radii,
        border_width: 0.0,
        z: Z_FLOAT,
        shadow_blur: 12.0,
        shadow_spread: 0.0,
        shadow_color: [0.0, 0.0, 0.0, 0.25],
        shadow_offset,
        _pad: [0.0; 2],
    });

    let close_requested = core.input.is_clicked(scrim_rect);

    core.draw_list.push_scissor(panel_rect);

    DrawerResponse { close_requested }
}

pub fn drawer_end(core: &mut AkarCore) {
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
    fn zero_width_renders_nothing() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let resp = drawer_begin(
            &mut core,
            [0.0, 0.0, 400.0, 600.0],
            DrawerEdge::Left,
            0.0,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
        assert!(!resp.close_requested);
        drawer_end(&mut core);
    }

    #[test]
    fn left_edge_renders_scrim_and_panel() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        drawer_begin(
            &mut core,
            [0.0, 0.0, 400.0, 600.0],
            DrawerEdge::Left,
            300.0,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 2);

        assert_eq!(quads[0].z, Z_SCRIM);
        assert_eq!(quads[0].rect[0], 300.0);
        assert_eq!(quads[0].rect[2], 100.0);

        assert_eq!(quads[1].z, Z_FLOAT);
        assert_eq!(quads[1].rect[0], 0.0);
        assert_eq!(quads[1].rect[2], 300.0);

        drawer_end(&mut core);
    }

    #[test]
    fn right_edge_renders_scrim_and_panel() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        drawer_begin(
            &mut core,
            [0.0, 0.0, 400.0, 600.0],
            DrawerEdge::Right,
            300.0,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 2);

        assert_eq!(quads[1].z, Z_FLOAT);
        assert_eq!(quads[1].rect[0], 100.0);
        assert_eq!(quads[1].rect[2], 300.0);

        drawer_end(&mut core);
    }

    #[test]
    fn scrim_click_requests_close() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        core.input.set_mouse_pos(350.0, 300.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.push_mouse_button(0, false);

        let resp = drawer_begin(
            &mut core,
            [0.0, 0.0, 400.0, 600.0],
            DrawerEdge::Left,
            300.0,
            &AKAR_THEME_DARK,
        );

        assert!(resp.close_requested);
        drawer_end(&mut core);
    }

    #[test]
    fn panel_click_does_not_request_close() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        core.input.set_mouse_pos(150.0, 300.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.push_mouse_button(0, false);

        let resp = drawer_begin(
            &mut core,
            [0.0, 0.0, 400.0, 600.0],
            DrawerEdge::Left,
            300.0,
            &AKAR_THEME_DARK,
        );

        assert!(!resp.close_requested);
        drawer_end(&mut core);
    }

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        drawer_begin(
            &mut core,
            [0.0, 0.0, 400.0, 600.0],
            DrawerEdge::Left,
            300.0,
            &AKAR_THEME_DARK,
        );
        assert!(core.draw_list.active_scissor().is_some());

        drawer_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }
}
