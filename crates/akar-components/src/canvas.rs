use glam::Vec2;
use akar_core::QuadCall;
use akar_layout::{Rect, CanvasTransform};
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
