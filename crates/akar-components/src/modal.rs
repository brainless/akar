use akar_core::{AkarCore, QuadCall, TextCall, Z_FLOAT, Z_SCRIM};
use akar_layout::{
    length, AlignItems, Dimension, Display, FlexDirection, Layout, NodeId, Size, Style,
};

use crate::color::color_to_f32;
use crate::AkarTheme;

#[derive(Clone, Copy, PartialEq)]
pub struct ModalResponse {
    pub close_requested: bool,
    pub content_node: NodeId,
    pub content_rect: [f32; 4],
    pub panel_rect: [f32; 4],
}

pub fn modal_begin(
    core: &mut AkarCore,
    layout: &mut Layout,
    viewport_rect: [f32; 4],
    title: &str,
    width: f32,
    height: f32,
    theme: &AkarTheme,
) -> ModalResponse {
    if width <= 0.0 || height <= 0.0 {
        return ModalResponse {
            close_requested: false,
            content_node: NodeId::new(0),
            content_rect: [0.0; 4],
            panel_rect: [0.0; 4],
        };
    }

    let [vx, vy, vw, vh] = viewport_rect;
    let clamped_w = width.min(vw);
    let clamped_h = height.min(vh);
    let panel_x = vx + (vw - clamped_w) / 2.0;
    let panel_y = vy + (vh - clamped_h) / 2.0;
    let panel_rect = [panel_x, panel_y, clamped_w, clamped_h];

    core.draw_list.push_quad(QuadCall {
        rect: viewport_rect,
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

    core.draw_list.push_quad(QuadCall {
        rect: panel_rect,
        fill: color_to_f32(theme.base_200),
        border_color: [0.0; 4],
        corner_radii: [theme.radius_box, 0.0, theme.radius_box, theme.radius_box],
        border_width: 0.0,
        z: Z_FLOAT,
        shadow_blur: 16.0,
        shadow_spread: 0.0,
        shadow_color: [0.0, 0.0, 0.0, 0.3],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let close_requested = core.input.is_clicked(viewport_rect);

    let header_height = 40.0;

    let title_node = layout.new_leaf(Style {
        flex_grow: 1.0,
        flex_shrink: 1.0,
        ..Default::default()
    });

    let close_node = layout.new_leaf(Style {
        flex_grow: 0.0,
        flex_shrink: 0.0,
        size: Size {
            width: length(32.0),
            height: Dimension::percent(1.0),
        },
        ..Default::default()
    });

    let header = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: Some(AlignItems::CENTER),
        size: Size {
            width: Dimension::percent(1.0),
            height: length(header_height),
        },
        ..Default::default()
    });
    layout.set_padding(header, 0.0, 8.0, 0.0, theme.padding_x);
    layout.set_children(header, &[title_node, close_node]);

    let content_node = layout.new_leaf(Style {
        display: Display::Flex,
        flex_grow: 1.0,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::auto(),
        },
        ..Default::default()
    });

    let column = layout.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        },
        &[header, content_node],
    );

    layout.compute(
        column,
        (Some(clamped_w), Some(clamped_h)),
        |_, _, _, _, _| akar_layout::Size::ZERO,
    );

    // Render header title
    let title_rect = layout.rect_offset(title_node, [panel_x, panel_y]);
    let title_buffer_id = core.text_pipeline.set_text(
        Some(title_node.into()),
        title,
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
        Some(title_rect[2]),
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: title_buffer_id,
        x: title_rect[0],
        y: title_rect[1],
        clip: title_rect,
        color: color_to_f32(theme.base_content),
        z: Z_FLOAT,
    });

    // Render close button
    let close_rect = layout.rect_offset(close_node, [panel_x, panel_y]);
    let close_buffer_id = core.text_pipeline.set_text(
        Some(close_node.into()),
        "\u{00d7}",
        glyphon::Metrics::new(theme.font_size_lg, theme.font_size_lg * 1.2),
        None,
        None,
    );
    core.draw_list.push_text(TextCall {
        buffer_id: close_buffer_id,
        x: close_rect[0],
        y: close_rect[1],
        clip: [close_rect[0], close_rect[1], 200.0, close_rect[3]],
        color: color_to_f32(theme.base_content),
        z: Z_FLOAT,
    });

    let close_clicked = core.input.is_clicked(close_rect);
    let close_requested = close_requested || close_clicked;

    let content_rect = layout.rect_offset(content_node, [panel_x, panel_y]);
    core.draw_list.push_scissor(content_rect);

    ModalResponse {
        close_requested,
        content_node,
        content_rect,
        panel_rect,
    }
}

pub fn modal_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;

    #[test]
    fn zero_width_height_returns_no_close() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let mut layout = Layout::new();

        let resp = modal_begin(
            &mut core,
            &mut layout,
            [0.0, 0.0, 400.0, 600.0],
            "Title",
            0.0,
            200.0,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
        assert!(!resp.close_requested);
    }

    #[test]
    fn renders_scrim_panel_and_header() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let mut layout = Layout::new();

        modal_begin(
            &mut core,
            &mut layout,
            [0.0, 0.0, 400.0, 600.0],
            "Test Modal",
            300.0,
            200.0,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert!(quads.len() >= 2, "expected >=2 quads, got {}", quads.len());
        assert_eq!(quads[0].z, Z_SCRIM);
        assert_eq!(quads[1].z, Z_FLOAT);
        assert!(
            core.draw_list.len() >= 4,
            "expected >=4 total calls, got {}",
            core.draw_list.len()
        );

        modal_end(&mut core);
    }

    #[test]
    fn scrim_click_requests_close() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let mut layout = Layout::new();

        core.input.set_mouse_pos(350.0, 300.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.push_mouse_button(0, false);

        let resp = modal_begin(
            &mut core,
            &mut layout,
            [0.0, 0.0, 400.0, 600.0],
            "Title",
            300.0,
            200.0,
            &AKAR_THEME_DARK,
        );

        assert!(resp.close_requested);
        modal_end(&mut core);
    }

    #[test]
    fn content_node_is_valid() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let mut layout = Layout::new();

        let resp = modal_begin(
            &mut core,
            &mut layout,
            [0.0, 0.0, 400.0, 600.0],
            "Title",
            300.0,
            200.0,
            &AKAR_THEME_DARK,
        );

        assert_ne!(resp.content_node, NodeId::new(0));
        modal_end(&mut core);
    }

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let mut layout = Layout::new();

        modal_begin(
            &mut core,
            &mut layout,
            [0.0, 0.0, 400.0, 600.0],
            "Title",
            300.0,
            200.0,
            &AKAR_THEME_DARK,
        );

        assert!(core.draw_list.active_scissor().is_some());

        modal_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }
}
