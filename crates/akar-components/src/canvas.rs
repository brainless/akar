use crate::color::color_to_f32;
use akar_core::{AkarCore, QuadCall, TextCall};
use akar_layout::{
    compute_visible_world_rect, make_screen_to_world, make_world_to_screen, CanvasTransform,
    Layout, NodeId, WorldRect,
};
use glam::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanButton {
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasConfig {
    pub pan_button: PanButton,
    pub zoom_sensitivity: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            pan_button: PanButton::Middle,
            zoom_sensitivity: 0.005,
            zoom_min: 0.1,
            zoom_max: 5.0,
        }
    }
}

pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub is_panning: bool,
}

impl CanvasState {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            is_panning: false,
        }
    }

    pub fn zoom_at_point(
        &mut self,
        screen_pos: Vec2,
        canvas_rect: [f32; 4],
        zoom_factor: f32,
        zoom_min: f32,
        zoom_max: f32,
    ) {
        let canvas_center = Vec2::new(
            canvas_rect[0] + canvas_rect[2] * 0.5,
            canvas_rect[1] + canvas_rect[3] * 0.5,
        );
        let world_pos = (screen_pos - canvas_center) / self.zoom + self.pan;
        let new_zoom = (self.zoom * zoom_factor).clamp(zoom_min, zoom_max);
        self.pan = world_pos - (screen_pos - canvas_center) / new_zoom;
        self.zoom = new_zoom;
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CanvasResponse {
    pub dragged: bool,
    pub zoomed: bool,
    pub world_to_screen: CanvasTransform,
    pub screen_to_world: CanvasTransform,
    pub visible_world_rect: WorldRect,
    canvas_rect: [f32; 4],
}

impl CanvasResponse {
    pub fn project(&self, world_rect: WorldRect) -> CanvasProjectedRect {
        let screen_rect = self.world_to_screen.apply_rect(world_rect);
        let canvas_rect = self.canvas_rect;
        let visible = screen_rect[0] < canvas_rect[0] + canvas_rect[2]
            && screen_rect[0] + screen_rect[2] > canvas_rect[0]
            && screen_rect[1] < canvas_rect[1] + canvas_rect[3]
            && screen_rect[1] + screen_rect[3] > canvas_rect[1];
        CanvasProjectedRect {
            screen_rect,
            pixels_per_world_unit: self.world_to_screen.scale,
            visible,
        }
    }

    pub fn lod_index(&self, world_rect: WorldRect, thresholds_px: &[f32]) -> usize {
        if thresholds_px.is_empty() {
            return 0;
        }
        let projected = self.project(world_rect);
        let min_dim = projected.screen_rect[2].min(projected.screen_rect[3]);
        for (i, &threshold) in thresholds_px.iter().enumerate() {
            if min_dim < threshold {
                return i;
            }
        }
        thresholds_px.len()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasProjectedRect {
    pub screen_rect: [f32; 4],
    pub pixels_per_world_unit: f32,
    pub visible: bool,
}

pub struct CanvasInput {
    pub world_mouse_pos: Vec2,
    pub mouse_buttons: [bool; 5],
    pub mouse_buttons_pressed: [bool; 5],
    pub mouse_buttons_released: [bool; 5],
}

impl CanvasInput {
    pub fn new(input: &akar_core::InputState, screen_to_world: &CanvasTransform) -> Self {
        Self {
            world_mouse_pos: screen_to_world.apply(input.mouse_pos),
            mouse_buttons: input.mouse_buttons,
            mouse_buttons_pressed: input.mouse_buttons_pressed,
            mouse_buttons_released: input.mouse_buttons_released,
        }
    }

    fn is_hovering_rect(&self, world_rect: WorldRect) -> bool {
        let p = self.world_mouse_pos;
        p.x >= world_rect.min.x
            && p.x <= world_rect.max.x
            && p.y >= world_rect.min.y
            && p.y <= world_rect.max.y
    }

    pub fn is_hovering(&self, world_rect: WorldRect) -> bool {
        self.is_hovering_rect(world_rect)
    }

    pub fn is_clicked(&self, world_rect: WorldRect) -> bool {
        self.mouse_buttons_released[0] && self.is_hovering_rect(world_rect)
    }

    pub fn is_pressed(&self, world_rect: WorldRect) -> bool {
        self.mouse_buttons[0] && self.is_hovering_rect(world_rect)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CanvasAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CanvasOverflow {
    Clip,
    Truncate,
}

pub struct CanvasTextStyle {
    pub font_size: f32,
    pub color: u32,
    pub background: Option<u32>,
    pub padding: [f32; 4],
    pub align_x: CanvasAlign,
    pub overflow: CanvasOverflow,
}

pub(crate) struct TextCallBuffer {
    world_rect: WorldRect,
    text: String,
    style: CanvasTextStyle,
}

pub struct CanvasPainter {
    pub(crate) quad_buffer: Vec<QuadCall>,
    pub(crate) text_buffer: Vec<TextCallBuffer>,
    pub(crate) world_to_screen: CanvasTransform,
    pub(crate) canvas_rect: [f32; 4],
}

impl CanvasPainter {
    pub fn push_quad(
        &mut self,
        world_rect: WorldRect,
        fill: u32,
        border_color: u32,
        border_width: f32,
        corner_radii: [f32; 4],
        z: f32,
    ) {
        let screen_rect = self.world_to_screen.apply_rect(world_rect);
        let scale = self.world_to_screen.scale;
        let scaled_radii = corner_radii.map(|r| self.world_to_screen.scale_radius(r));
        self.quad_buffer.push(QuadCall {
            rect: screen_rect,
            fill: color_to_f32(fill),
            border_color: color_to_f32(border_color),
            border_width: border_width * scale,
            corner_radii: scaled_radii,
            z,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    pub fn push_text(&mut self, world_rect: WorldRect, text: &str, style: &CanvasTextStyle) {
        self.text_buffer.push(TextCallBuffer {
            world_rect,
            text: text.to_string(),
            style: CanvasTextStyle {
                font_size: style.font_size,
                color: style.color,
                background: style.background,
                padding: style.padding,
                align_x: style.align_x,
                overflow: style.overflow,
            },
        });
    }
}

pub fn canvas_begin(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    state: &mut CanvasState,
    config: &CanvasConfig,
) -> (CanvasResponse, CanvasPainter) {
    let rect = layout.rect(node_id);

    core.draw_list.push_scissor(rect);

    let pan_btn = match config.pan_button {
        PanButton::Middle => 2,
        PanButton::Right => 1,
    };

    if core.input.mouse_buttons_pressed[pan_btn] && core.input.is_hovering(rect) {
        state.is_panning = true;
    }
    if !core.input.mouse_buttons[pan_btn] {
        state.is_panning = false;
    }

    let mut dragged = false;
    if state.is_panning {
        let delta = (core.input.mouse_pos - core.input.mouse_pos_prev) / state.zoom;
        if delta != Vec2::ZERO {
            state.pan -= delta;
            dragged = true;
        }
    }

    let mut zoomed = false;
    let scroll_y = core.input.scroll_delta.y;
    if scroll_y != 0.0 && core.input.is_hovering(rect) {
        let zoom_factor = 1.0 + scroll_y * config.zoom_sensitivity;
        if zoom_factor > 0.0 {
            state.zoom_at_point(
                core.input.mouse_pos,
                rect,
                zoom_factor,
                config.zoom_min,
                config.zoom_max,
            );
            zoomed = true;
        }
    }

    let world_to_screen = make_world_to_screen(state.pan, state.zoom, rect);
    let screen_to_world = make_screen_to_world(state.pan, state.zoom, rect);
    let visible_world_rect = compute_visible_world_rect(state.pan, state.zoom, rect);

    let response = CanvasResponse {
        dragged,
        zoomed,
        world_to_screen,
        screen_to_world,
        visible_world_rect,
        canvas_rect: rect,
    };
    let painter = CanvasPainter {
        quad_buffer: Vec::new(),
        text_buffer: Vec::new(),
        world_to_screen,
        canvas_rect: rect,
    };

    (response, painter)
}

pub fn canvas_end(core: &mut AkarCore, painter: CanvasPainter) {
    for quad in painter.quad_buffer {
        core.draw_list.push_quad(quad);
    }

    let canvas_screen_rect = painter.canvas_rect;
    let w2s = painter.world_to_screen;
    let zoom = w2s.scale;

    for (i, entry) in painter.text_buffer.iter().enumerate() {
        let screen_rect = w2s.apply_rect(entry.world_rect);
        let visible = screen_rect[0] < canvas_screen_rect[0] + canvas_screen_rect[2]
            && screen_rect[0] + screen_rect[2] > canvas_screen_rect[0]
            && screen_rect[1] < canvas_screen_rect[1] + canvas_screen_rect[3]
            && screen_rect[1] + screen_rect[3] > canvas_screen_rect[1];
        if !visible {
            continue;
        }

        let font_size_px = entry.style.font_size * zoom;
        let line_height = font_size_px * 1.2;
        let metrics = glyphon::Metrics::new(font_size_px, line_height);

        let buffer_id = u64::MAX - i as u64;
        let padding = entry.style.padding;
        let pad_left = padding[3] * zoom;
        let pad_top = padding[0] * zoom;
        let pad_right = padding[1] * zoom;
        let pad_bottom = padding[2] * zoom;

        let text_area_w = (screen_rect[2] - pad_left - pad_right).max(0.0);
        let text_area_h = (screen_rect[3] - pad_top - pad_bottom).max(0.0);
        if text_area_w <= 0.0 || text_area_h <= 0.0 {
            continue;
        }

        core.text_pipeline
            .set_text(Some(buffer_id), &entry.text, metrics, Some(text_area_w), None);

        let text_size = core.text_pipeline.measure(buffer_id, Some(text_area_w));

        let text_x = match entry.style.align_x {
            CanvasAlign::Left => screen_rect[0] + pad_left,
            CanvasAlign::Center => {
                screen_rect[0] + pad_left + (text_area_w - text_size.x) * 0.5
            }
            CanvasAlign::Right => screen_rect[0] + screen_rect[2] - pad_right - text_size.x,
        };
        let text_y = screen_rect[1] + pad_top;

        let color = color_to_f32(entry.style.color);

        if let Some(bg) = entry.style.background {
            let bg_call = QuadCall {
                rect: screen_rect,
                fill: color_to_f32(bg),
                border_color: [0.0; 4],
                border_width: 0.0,
                corner_radii: [0.0; 4],
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            };
            core.draw_list.push_quad(bg_call);
        }

        let clip = match entry.style.overflow {
            CanvasOverflow::Clip => screen_rect,
            CanvasOverflow::Truncate => {
                let max_h = text_size.y.min(text_area_h);
                [screen_rect[0], screen_rect[1], screen_rect[2], pad_top + max_h]
            }
        };

        core.draw_list.push_text(TextCall {
            buffer_id,
            x: text_x,
            y: text_y,
            clip,
            color,
            z: 0.0,
        });
    }

    core.draw_list.pop_scissor();
}

pub struct CanvasPortalGuard {
    pub screen_rect: [f32; 4],
}

/// Push a scissor rect for a portal's projected screen bounds.
///
/// Ordering: render buffered canvas content (`canvas_end`) before calling
/// this. Render the portal subtree between begin and end. The portal's
/// scissor composes with any existing canvas scissor via intersection.
pub fn canvas_portal_begin(
    core: &mut AkarCore,
    portal_layout: &Layout,
    root_node: NodeId,
) -> CanvasPortalGuard {
    let screen_rect = portal_layout.rect(root_node);
    core.draw_list.push_scissor(screen_rect);
    CanvasPortalGuard { screen_rect }
}

pub fn canvas_portal_end(core: &mut AkarCore, _guard: CanvasPortalGuard) {
    core.draw_list.pop_scissor();
}

pub fn is_visible_world(viewport: WorldRect, target: WorldRect) -> bool {
    viewport.intersects(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::InputState;
    use glam::Vec2;

    const CANVAS: [f32; 4] = [0.0, 0.0, 800.0, 600.0];

    fn make_response(pan: Vec2, zoom: f32) -> CanvasResponse {
        let w2s = make_world_to_screen(pan, zoom, CANVAS);
        let s2w = make_screen_to_world(pan, zoom, CANVAS);
        let visible = compute_visible_world_rect(pan, zoom, CANVAS);
        CanvasResponse {
            dragged: false,
            zoomed: false,
            world_to_screen: w2s,
            screen_to_world: s2w,
            visible_world_rect: visible,
            canvas_rect: CANVAS,
        }
    }

    #[test]
    fn zoom_at_point_anchors_cursor() {
        let mut state = CanvasState::new();
        let cursor = Vec2::new(500.0, 300.0);
        let canvas_center = Vec2::new(400.0, 300.0);
        let world_before = (cursor - canvas_center) / state.zoom + state.pan;
        state.zoom_at_point(cursor, CANVAS, 2.0, 0.1, 5.0);
        let screen_after = (world_before - state.pan) * state.zoom + canvas_center;
        assert!(
            (screen_after - cursor).length() < 0.001,
            "got {screen_after}"
        );
    }

    #[test]
    fn zoom_clamps_at_min() {
        let mut state = CanvasState {
            pan: Vec2::ZERO,
            zoom: 0.15,
            is_panning: false,
        };
        state.zoom_at_point(Vec2::new(400.0, 300.0), CANVAS, 0.1, 0.1, 5.0);
        assert!(state.zoom >= 0.1);
    }

    #[test]
    fn zoom_clamps_at_max() {
        let mut state = CanvasState {
            pan: Vec2::ZERO,
            zoom: 4.9,
            is_panning: false,
        };
        state.zoom_at_point(Vec2::new(400.0, 300.0), CANVAS, 10.0, 0.1, 5.0);
        assert!(state.zoom <= 5.0);
    }

    #[test]
    fn is_visible_world_cases() {
        let viewport = WorldRect {
            min: Vec2::new(-100.0, -100.0),
            max: Vec2::new(100.0, 100.0),
        };
        let inside = WorldRect {
            min: Vec2::new(-50.0, -50.0),
            max: Vec2::new(50.0, 50.0),
        };
        let outside = WorldRect {
            min: Vec2::new(200.0, 200.0),
            max: Vec2::new(300.0, 300.0),
        };
        let touching = WorldRect {
            min: Vec2::new(100.0, -50.0),
            max: Vec2::new(200.0, 50.0),
        };
        let partial = WorldRect {
            min: Vec2::new(50.0, 50.0),
            max: Vec2::new(150.0, 150.0),
        };
        assert!(is_visible_world(viewport, inside));
        assert!(!is_visible_world(viewport, outside));
        assert!(is_visible_world(viewport, touching));
        assert!(is_visible_world(viewport, partial));
    }

    #[test]
    fn push_quad_transforms_rect() {
        let w2s = akar_layout::make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let mut painter = CanvasPainter {
            quad_buffer: Vec::new(),
            text_buffer: Vec::new(),
            world_to_screen: w2s,
            canvas_rect: CANVAS,
        };
        let world_rect = WorldRect {
            min: Vec2::new(-5.0, -5.0),
            max: Vec2::new(5.0, 5.0),
        };
        painter.push_quad(world_rect, 0xFF0000FF, 0x00000000, 0.0, [0.0; 4], 0.0);
        assert_eq!(painter.quad_buffer.len(), 1);
        let [x, y, w, h] = painter.quad_buffer[0].rect;
        assert!((x - 390.0).abs() < 0.001, "x={x}");
        assert!((y - 290.0).abs() < 0.001, "y={y}");
        assert!((w - 20.0).abs() < 0.001, "w={w}");
        assert!((h - 20.0).abs() < 0.001, "h={h}");
    }

    #[test]
    fn projected_rect_basic() {
        let resp = make_response(Vec2::ZERO, 1.0);
        let world_rect = WorldRect::from_xywh(-5.0, -5.0, 10.0, 10.0);
        let pr = resp.project(world_rect);
        assert!((pr.screen_rect[0] - 395.0).abs() < 0.001);
        assert!((pr.screen_rect[1] - 295.0).abs() < 0.001);
        assert!((pr.screen_rect[2] - 10.0).abs() < 0.001);
        assert!((pr.screen_rect[3] - 10.0).abs() < 0.001);
        assert!((pr.pixels_per_world_unit - 1.0).abs() < 0.001);
        assert!(pr.visible);
    }

    #[test]
    fn projected_rect_not_visible() {
        let resp = make_response(Vec2::ZERO, 1.0);
        let world_rect = WorldRect::from_xywh(5000.0, 5000.0, 10.0, 10.0);
        let pr = resp.project(world_rect);
        assert!(!pr.visible);
    }

    #[test]
    fn lod_index_empty_thresholds() {
        let resp = make_response(Vec2::ZERO, 1.0);
        let world_rect = WorldRect::from_xywh(-5.0, -5.0, 10.0, 10.0);
        assert_eq!(resp.lod_index(world_rect, &[]), 0);
    }

    #[test]
    fn lod_index_below_all() {
        let resp = make_response(Vec2::ZERO, 1.0);
        let world_rect = WorldRect::from_xywh(-1.0, -1.0, 2.0, 2.0);
        assert_eq!(resp.lod_index(world_rect, &[10.0, 50.0, 100.0]), 0);
    }

    #[test]
    fn lod_index_at_boundary() {
        let resp = make_response(Vec2::ZERO, 1.0);
        let world_rect = WorldRect::from_xywh(-5.0, -5.0, 10.0, 10.0);
        assert_eq!(resp.lod_index(world_rect, &[10.0, 50.0, 100.0]), 1);
    }

    #[test]
    fn lod_index_above_all() {
        let resp = make_response(Vec2::ZERO, 2.0);
        let world_rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        assert_eq!(resp.lod_index(world_rect, &[10.0, 50.0, 100.0]), 3);
    }

    #[test]
    fn canvas_input_hover() {
        let s2w = make_screen_to_world(Vec2::ZERO, 1.0, CANVAS);
        let mut input = InputState::new();
        input.set_mouse_pos(400.0, 300.0);
        let ci = CanvasInput::new(&input, &s2w);
        let world_rect = WorldRect::from_xywh(-10.0, -10.0, 20.0, 20.0);
        assert!(ci.is_hovering(world_rect));
    }

    #[test]
    fn canvas_input_click() {
        let s2w = make_screen_to_world(Vec2::ZERO, 1.0, CANVAS);
        let mut input = InputState::new();
        input.set_mouse_pos(400.0, 300.0);
        input.push_mouse_button(0, true);
        input.begin_frame();
        input.push_mouse_button(0, false);
        let ci = CanvasInput::new(&input, &s2w);
        let world_rect = WorldRect::from_xywh(-10.0, -10.0, 20.0, 20.0);
        assert!(ci.is_clicked(world_rect));
    }

    #[test]
    fn push_quad_scales_border_width() {
        let w2s = make_world_to_screen(Vec2::ZERO, 3.0, CANVAS);
        let mut painter = CanvasPainter {
            quad_buffer: Vec::new(),
            text_buffer: Vec::new(),
            world_to_screen: w2s,
            canvas_rect: CANVAS,
        };
        let world_rect = WorldRect::from_xywh(-5.0, -5.0, 10.0, 10.0);
        painter.push_quad(world_rect, 0xFF0000FF, 0x00000000, 2.0, [0.0; 4], 0.0);
        assert!((painter.quad_buffer[0].border_width - 6.0).abs() < 0.001);
    }

    #[test]
    fn push_quad_scales_shadow_fields() {
        let w2s = make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let mut painter = CanvasPainter {
            quad_buffer: Vec::new(),
            text_buffer: Vec::new(),
            world_to_screen: w2s,
            canvas_rect: CANVAS,
        };
        let world_rect = WorldRect::from_xywh(-5.0, -5.0, 10.0, 10.0);
        painter.push_quad(world_rect, 0xFF0000FF, 0x00000000, 2.0, [1.0, 2.0, 3.0, 4.0], 0.0);
        let q = &painter.quad_buffer[0];
        assert!((q.corner_radii[0] - 2.0).abs() < 0.001);
        assert!((q.corner_radii[1] - 4.0).abs() < 0.001);
        assert!((q.corner_radii[2] - 6.0).abs() < 0.001);
        assert!((q.corner_radii[3] - 8.0).abs() < 0.001);
    }

    #[test]
    fn canvas_text_invisible_culled() {
        let mut core = AkarCore::mock();
        core.begin_frame(800, 600, 1.0);
        let w2s = make_world_to_screen(Vec2::ZERO, 1.0, CANVAS);

        let mut painter = CanvasPainter {
            quad_buffer: Vec::new(),
            text_buffer: Vec::new(),
            world_to_screen: w2s,
            canvas_rect: CANVAS,
        };

        let style = CanvasTextStyle {
            font_size: 12.0,
            color: 0xFFFFFFFF,
            background: None,
            padding: [0.0; 4],
            align_x: CanvasAlign::Left,
            overflow: CanvasOverflow::Clip,
        };

        let far_away = WorldRect::from_xywh(5000.0, 5000.0, 10.0, 10.0);
        painter.push_text(far_away, "invisible", &style);

        canvas_end(&mut core, painter);

        let text_calls: Vec<_> = core
            .draw_list
            .text_calls()
            .iter()
            .filter_map(|c| match c {
                akar_core::DrawCall::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(text_calls.is_empty(), "invisible text should be culled");
    }

    #[test]
    fn canvas_text_no_widget_state() {
        let mut core = AkarCore::mock();
        core.begin_frame(800, 600, 1.0);
        let w2s = make_world_to_screen(Vec2::ZERO, 1.0, CANVAS);

        let mut painter = CanvasPainter {
            quad_buffer: Vec::new(),
            text_buffer: Vec::new(),
            world_to_screen: w2s,
            canvas_rect: CANVAS,
        };

        let style = CanvasTextStyle {
            font_size: 12.0,
            color: 0xFFFFFFFF,
            background: None,
            padding: [0.0; 4],
            align_x: CanvasAlign::Left,
            overflow: CanvasOverflow::Clip,
        };

        let world_rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        painter.push_text(world_rect, "hello", &style);

        canvas_end(&mut core, painter);

        assert!(
            core.input.focused_id.is_none(),
            "canvas text must not set focused_id"
        );
    }

    fn make_portal_layout(rect: [f32; 4]) -> (Layout, NodeId) {
        let mut layout = Layout::new();
        layout.set_screen_origin([rect[0], rect[1]]);
        let root = layout.new_leaf(akar_layout::Style {
            size: akar_layout::Size {
                width: akar_layout::length(rect[2]),
                height: akar_layout::length(rect[3]),
            },
            ..Default::default()
        });
        layout.compute(root, (Some(rect[2]), Some(rect[3])), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        (layout, root)
    }

    #[test]
    fn portal_pushes_scissor() {
        let mut core = AkarCore::mock();
        core.begin_frame(800, 600, 1.0);

        let (layout, root) = make_portal_layout([100.0, 50.0, 300.0, 200.0]);
        let guard = canvas_portal_begin(&mut core, &layout, root);

        let scissor = core.draw_list.active_scissor().unwrap();
        assert!(
            (scissor[0] - 100.0).abs() < 0.001,
            "scissor.x={}",
            scissor[0]
        );
        assert!(
            (scissor[1] - 50.0).abs() < 0.001,
            "scissor.y={}",
            scissor[1]
        );
        assert!(
            (scissor[2] - 300.0).abs() < 0.001,
            "scissor.w={}",
            scissor[2]
        );
        assert!(
            (scissor[3] - 200.0).abs() < 0.001,
            "scissor.h={}",
            scissor[3]
        );
        assert_eq!(guard.screen_rect, [100.0, 50.0, 300.0, 200.0]);

        canvas_portal_end(&mut core, guard);
    }

    #[test]
    fn portal_end_pops_scissor() {
        let mut core = AkarCore::mock();
        core.begin_frame(800, 600, 1.0);

        let (layout, root) = make_portal_layout([100.0, 50.0, 300.0, 200.0]);
        let guard = canvas_portal_begin(&mut core, &layout, root);
        assert!(core.draw_list.active_scissor().is_some());

        canvas_portal_end(&mut core, guard);
        assert!(
            core.draw_list.active_scissor().is_none(),
            "scissor should be popped after portal_end"
        );
    }

    #[test]
    fn portal_nests_with_canvas_scissor() {
        let mut core = AkarCore::mock();
        core.begin_frame(800, 600, 1.0);

        core.draw_list.push_scissor([0.0, 0.0, 800.0, 600.0]);

        let (layout, root) = make_portal_layout([100.0, 50.0, 300.0, 200.0]);
        let guard = canvas_portal_begin(&mut core, &layout, root);

        let scissor = core.draw_list.active_scissor().unwrap();
        assert!(
            (scissor[0] - 100.0).abs() < 0.001,
            "intersection x={}",
            scissor[0]
        );
        assert!(
            (scissor[1] - 50.0).abs() < 0.001,
            "intersection y={}",
            scissor[1]
        );
        assert!(
            (scissor[2] - 300.0).abs() < 0.001,
            "intersection w={}",
            scissor[2]
        );
        assert!(
            (scissor[3] - 200.0).abs() < 0.001,
            "intersection h={}",
            scissor[3]
        );

        canvas_portal_end(&mut core, guard);

        let restored = core.draw_list.active_scissor().unwrap();
        assert_eq!(restored, [0.0, 0.0, 800.0, 600.0]);

        core.draw_list.pop_scissor();
    }

    #[test]
    fn portal_zero_area() {
        let mut core = AkarCore::mock();
        core.begin_frame(800, 600, 1.0);

        let (layout, root) = make_portal_layout([200.0, 100.0, 0.0, 0.0]);
        let guard = canvas_portal_begin(&mut core, &layout, root);

        let scissor = core.draw_list.active_scissor().unwrap();
        assert_eq!(scissor[2], 0.0, "zero-width scissor");
        assert_eq!(scissor[3], 0.0, "zero-height scissor");

        canvas_portal_end(&mut core, guard);
    }
}
