use glam::Vec2;
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Rect, CanvasTransform, Layout, NodeId, make_world_to_screen, make_screen_to_world, compute_visible_world_rect};
use crate::color::color_to_f32;

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
        Self { pan: Vec2::ZERO, zoom: 1.0, is_panning: false }
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
    fn default() -> Self { Self::new() }
}

pub struct CanvasResponse {
    pub dragged: bool,
    pub zoomed: bool,
    pub world_to_screen: CanvasTransform,
    pub screen_to_world: CanvasTransform,
    pub visible_world_rect: Rect,
}

pub struct CanvasPainter {
    pub(crate) buffer: Vec<QuadCall>,
    pub(crate) world_to_screen: CanvasTransform,
}

impl CanvasPainter {
    pub fn push_quad(
        &mut self,
        world_rect: Rect,
        fill: u32,
        border_color: u32,
        border_width: f32,
        corner_radii: [f32; 4],
        z: f32,
    ) {
        let screen_rect = self.world_to_screen.apply_rect(world_rect);
        let scaled_radii = corner_radii.map(|r| self.world_to_screen.scale_radius(r));
        self.buffer.push(QuadCall {
            rect: screen_rect,
            fill: color_to_f32(fill),
            border_color: color_to_f32(border_color),
            border_width,
            corner_radii: scaled_radii,
            z,
            _pad: [0.0; 2],
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

    let response = CanvasResponse { dragged, zoomed, world_to_screen, screen_to_world, visible_world_rect };
    let painter = CanvasPainter { buffer: Vec::new(), world_to_screen };

    (response, painter)
}

pub fn canvas_end(core: &mut AkarCore, painter: CanvasPainter) {
    for quad in painter.buffer {
        core.draw_list.push_quad(quad);
    }
    core.draw_list.pop_scissor();
}

pub fn is_visible_world(viewport: Rect, target: Rect) -> bool {
    viewport.intersects(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    const CANVAS: [f32; 4] = [0.0, 0.0, 800.0, 600.0];

    #[test]
    fn zoom_at_point_anchors_cursor() {
        let mut state = CanvasState::new();
        let cursor = Vec2::new(500.0, 300.0);
        let canvas_center = Vec2::new(400.0, 300.0);
        let world_before = (cursor - canvas_center) / state.zoom + state.pan;
        state.zoom_at_point(cursor, CANVAS, 2.0, 0.1, 5.0);
        let screen_after = (world_before - state.pan) * state.zoom + canvas_center;
        assert!((screen_after - cursor).length() < 0.001, "got {screen_after}");
    }

    #[test]
    fn zoom_clamps_at_min() {
        let mut state = CanvasState { pan: Vec2::ZERO, zoom: 0.15, is_panning: false };
        state.zoom_at_point(Vec2::new(400.0, 300.0), CANVAS, 0.1, 0.1, 5.0);
        assert!(state.zoom >= 0.1);
    }

    #[test]
    fn zoom_clamps_at_max() {
        let mut state = CanvasState { pan: Vec2::ZERO, zoom: 4.9, is_panning: false };
        state.zoom_at_point(Vec2::new(400.0, 300.0), CANVAS, 10.0, 0.1, 5.0);
        assert!(state.zoom <= 5.0);
    }

    #[test]
    fn is_visible_world_cases() {
        let viewport = Rect { min: Vec2::new(-100.0, -100.0), max: Vec2::new(100.0, 100.0) };
        let inside   = Rect { min: Vec2::new(-50.0, -50.0),   max: Vec2::new(50.0, 50.0) };
        let outside  = Rect { min: Vec2::new(200.0, 200.0),   max: Vec2::new(300.0, 300.0) };
        let touching = Rect { min: Vec2::new(100.0, -50.0),   max: Vec2::new(200.0, 50.0) };
        let partial  = Rect { min: Vec2::new(50.0, 50.0),     max: Vec2::new(150.0, 150.0) };
        assert!(is_visible_world(viewport, inside));
        assert!(!is_visible_world(viewport, outside));
        assert!(is_visible_world(viewport, touching));
        assert!(is_visible_world(viewport, partial));
    }

    #[test]
    fn push_quad_transforms_rect() {
        let w2s = akar_layout::make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let mut painter = CanvasPainter { buffer: Vec::new(), world_to_screen: w2s };
        let world_rect = Rect { min: Vec2::new(-5.0, -5.0), max: Vec2::new(5.0, 5.0) };
        painter.push_quad(world_rect, 0xFF0000FF, 0x00000000, 0.0, [0.0; 4], 0.0);
        assert_eq!(painter.buffer.len(), 1);
        let [x, y, w, h] = painter.buffer[0].rect;
        assert!((x - 390.0).abs() < 0.001, "x={x}");
        assert!((y - 290.0).abs() < 0.001, "y={y}");
        assert!((w - 20.0).abs() < 0.001,  "w={w}");
        assert!((h - 20.0).abs() < 0.001,  "h={h}");
    }
}
